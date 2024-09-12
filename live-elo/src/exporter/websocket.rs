use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use lbo::exporter::Exporter;
use serde::Serialize;
use tokio::sync::{broadcast, mpsc, RwLock};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, instrument, trace};

use crate::sources::AuthorId;

use super::elo_calculator::EloProcessor;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct LeaderboardName(&'static str);

impl LeaderboardName {
    pub fn new(name: &'static str) -> Self {
        Self(name)
    }

    pub fn get(&self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PerformancePoints(f32);

impl PerformancePoints {
    pub fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn get(&self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Elo(f32);

impl Elo {
    pub fn new(value: f32) -> Self {
        if !value.is_finite() {
            error!(?value, "elo value was not finite");
            panic!("elo value was not finite: {value:?}");
        }
        Self(value)
    }

    pub fn get(&self) -> f32 {
        self.0
    }
}

// TODO: make this into a newtype smhsmh
pub type LeaderboardPerformances = HashMap<AuthorId, PerformancePoints>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LeaderboardEloEntry {
    pub author_id: AuthorId,
    pub elo: Elo,
}

impl From<LeaderboardStateEntry> for LeaderboardEloEntry {
    fn from(value: LeaderboardStateEntry) -> Self {
        Self {
            author_id: value.author_id,
            elo: value.elo,
        }
    }
}

pub type LeaderboardElos = Vec<LeaderboardEloEntry>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LeaderboardStateEntry {
    pub author_id: AuthorId,
    pub elo: Elo,
    pub performance_points: PerformancePoints,
}

pub type LeaderboardStates = Vec<LeaderboardStateEntry>;

type LeaderboardPosition = usize;
type LeaderboardElosChanges = HashMap<LeaderboardPosition, LeaderboardEloEntry>;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
struct LeaderboardsChanges(HashMap<LeaderboardName, LeaderboardElosChanges>);

impl LeaderboardsChanges {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self) -> &HashMap<LeaderboardName, LeaderboardElosChanges> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut HashMap<LeaderboardName, LeaderboardElosChanges> {
        &mut self.0
    }
}

#[instrument(level = "trace", skip_all)]
pub async fn turn_batched_performances_into_real_leaderboards(
    mut incoming: mpsc::Receiver<FullBatchedPerformances>,
    // base_leaderboards: HashMap<LeaderboardName, LeaderboardElos>,
    leaderboards: Arc<RwLock<HashMap<LeaderboardName, LeaderboardElos>>>,
    outgoing: mpsc::Sender<LeaderboardsChanges>,
) {
    let base_leaderboards = leaderboards.read().await;
    let elo_calculators: HashMap<LeaderboardName, EloProcessor> = HashMap::from_iter(
        base_leaderboards
            .clone()
            .into_iter()
            .map(|(key, value)| (key, EloProcessor::new(value))),
    );
    let mut all_current_performances: HashMap<LeaderboardName, LeaderboardPerformances> =
        HashMap::from_iter(
            base_leaderboards
                .keys()
                .map(|key| (key.to_owned(), LeaderboardPerformances::new())),
        );
    drop(base_leaderboards);

    while let Some(mut batch) = incoming.recv().await {
        trace!("got new batch");
        let mut current_leaderboards = leaderboards.write().await;
        let before = current_leaderboards.clone();

        for (leaderboard_name, leaderboard) in current_leaderboards.iter_mut() {
            if let Some(changes) = batch.remove(leaderboard_name) {
                let current_performances =
                    all_current_performances.get_mut(leaderboard_name).unwrap();
                for (author, delta) in changes {
                    let score = current_performances
                        .entry(author)
                        .or_insert(PerformancePoints::new(0.0));
                    *score = PerformancePoints::new(score.get() + delta.get());
                }

                // This is a bit roundabout but I couldn't figure out how to let tokio's spawn_blocking
                // hold a reference to a local reference nicely
                // The important thing here is moving `EloProcessor::run()` outside of the async function
                // so it doesn't block for a (relatively) long time
                let (leaderboard_send, leaderboard_recv) = tokio::sync::oneshot::channel();
                std::thread::scope(|s| {
                    s.spawn(|| {
                        let new_leaderboard =
                            elo_calculators[leaderboard_name].run(current_performances);
                        leaderboard_send.send(new_leaderboard).unwrap();
                    });
                });

                let new_leaderboard = leaderboard_recv.await.unwrap();
                *leaderboard = new_leaderboard;
            }
        }

        trace!("finding changes");
        let delta = find_changes(&before, &current_leaderboards);
        trace!("found changes");
        outgoing.send(delta).await.unwrap();
    }
}

type LeaderboardPerformancesDelta = HashMap<AuthorId, PerformancePoints>;
#[derive(Debug, Clone)]
struct IngestedPerformance {
    pub leaderboard_name: LeaderboardName,
    pub author_id: AuthorId,
    pub performance: PerformancePoints,
}

type FullBatchedPerformances = HashMap<LeaderboardName, LeaderboardPerformancesDelta>;

#[instrument(level = "trace", skip_all)]
pub async fn batch_performance_updates(
    mut incoming: mpsc::Receiver<IngestedPerformance>,
    outgoing: mpsc::Sender<FullBatchedPerformances>,
) {
    loop {
        let first = incoming.recv().await;
        trace!(?first, "got first message in batch");

        if let None = first {
            break;
        }

        let first = first.unwrap();
        let mut read = Vec::new();
        read.push(first);

        let _ = tokio::time::timeout(
            // FIXME: raise this probably
            Duration::from_secs(5),
            read_as_many_as_possible(&mut incoming, &mut read),
        )
        .await;

        let before_size = read.len();
        let batch = squash_batch(read);
        trace!(
            new_size = batch.len(),
            old_size = before_size,
            "squashed batch"
        );

        outgoing.send(batch).await.unwrap();
    }

    debug!("batch_performance_updates loop exited")
}

fn squash_batch(read: Vec<IngestedPerformance>) -> FullBatchedPerformances {
    let mut batch = FullBatchedPerformances::new();

    for IngestedPerformance {
        leaderboard_name,
        author_id,
        performance,
    } in read
    {
        let leaderboard = batch.entry(leaderboard_name).or_default();
        let author_score = leaderboard
            .entry(author_id)
            .or_insert(PerformancePoints::new(0.0));
        *author_score = PerformancePoints::new(author_score.get() + performance.get());
    }

    batch
}

async fn read_as_many_as_possible<T>(mpsc: &mut mpsc::Receiver<T>, into: &mut Vec<T>) {
    while let Some(value) = mpsc.recv().await {
        into.push(value);
    }
}

fn find_changes(
    from: &HashMap<LeaderboardName, LeaderboardElos>,
    to: &HashMap<LeaderboardName, LeaderboardElos>,
) -> LeaderboardsChanges {
    let mut changes = LeaderboardsChanges::new();
    for (name, before) in from {
        let leaderboard_changes = changes.get_mut().entry(name.to_owned()).or_default();
        let now = to.get(name).unwrap();

        for (index, now_at) in now.into_iter().enumerate() {
            if before
                .get(index)
                .map(|b| b != now_at)
                .unwrap_or(true)
            {
                leaderboard_changes.insert(index, now_at.to_owned().into());
            }
        }
    }

    changes
}

pub struct WebServerHandle {
    server_task: tokio::task::JoinHandle<Result<(), std::io::Error>>,
    dependent_tasks: tokio::task::JoinSet<()>,
    state: WebState,
    // This needs to be here because otherwise the channel will close
    serialized_recv: broadcast::Receiver<Arc<SerializedOutgoingMessage>>,
}

impl WebServerHandle {
    pub async fn close(mut self) {
        drop(self.state);
        drop(self.serialized_recv);
        // TODO: there might be a better way to gracefully shut down the web server?
        self.server_task.abort();

        while let Some(result) = self.dependent_tasks.join_next().await {
            result.unwrap();
        }
    }
}

async fn run_webserver(
    ingest_recv: mpsc::Receiver<IngestedPerformance>,
    leaderboards_states: HashMap<LeaderboardName, LeaderboardElos>,
) -> WebServerHandle {
    let (serialized_send, serialized_recv) = broadcast::channel(10);
    let current_leaderboards_states = Arc::new(RwLock::new(leaderboards_states));
    let state = WebState {
        serialized_send: serialized_send.clone(),
        current_leaderboards_states: current_leaderboards_states.clone(),
    };

    let mut dependent_tasks = tokio::task::JoinSet::new();
    let (batched_send, batched_recv) = mpsc::channel(10_000);
    let (changes_send, changes_recv) = mpsc::channel(10_000);
    dependent_tasks.spawn(batch_performance_updates(ingest_recv, batched_send));
    dependent_tasks.spawn(turn_batched_performances_into_real_leaderboards(
        batched_recv,
        current_leaderboards_states.clone(),
        changes_send,
    ));
    dependent_tasks.spawn(serialize_changes(changes_recv, serialized_send));

    let router = Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/websocket", get(get_websocket))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("localhost:8000")
        .await
        .unwrap();
    let webserver_task = tokio::task::spawn(async move {
        let local_addr = listener.local_addr().unwrap();
        info!(?local_addr, "starting listener");
        axum::serve(listener, router).await
    });

    WebServerHandle {
        server_task: webserver_task,
        dependent_tasks,
        state,
        serialized_recv,
    }
}

async fn get_websocket(ws: WebSocketUpgrade, State(state): State<WebState>) -> impl IntoResponse {
    // TODO: make sure that these limits are only applied to incoming messages, and if so
    //       they could probably be lowered further
    ws.max_frame_size(2048)
        .max_message_size(2048)
        .on_upgrade(move |ws| handle_websocket(ws, state))
}

#[derive(Clone)]
struct WebState {
    serialized_send: broadcast::Sender<Arc<SerializedOutgoingMessage>>,
    current_leaderboards_states: Arc<RwLock<HashMap<LeaderboardName, LeaderboardElos>>>,
}

async fn handle_websocket(mut ws: WebSocket, state: WebState) {
    let initial_state_lock = state.current_leaderboards_states.read().await;
    // This needs to happen whilst we have the read guard, as otherwise there's a risk
    // that we get a batch that has already been applied.
    // I don't think that would actually cause an issue come to think of it, but I'm unsure
    let mut batch_updater_recv = state.serialized_send.subscribe();
    let initial_state = initial_state_lock.clone();
    drop(initial_state_lock);
    // TODO: this should probably be stored somewhere to mitigate the effect on CPU usage if
    //       many clients connect at once
    let initial_state_serialized = serde_json::to_vec(&OutgoingMessage::InitialLeaderboards {
        leaderboards: initial_state
            .into_iter()
            .map(|(key, value)| (key, value.into_iter().map(|entry| entry.into()).collect()))
            .collect(),
    })
    .unwrap();
    ws.send(axum::extract::ws::Message::Binary(initial_state_serialized))
        .await
        .unwrap();

    loop {
        let message = tokio::select! {
            message = ws.recv() => message.map(|message| WebsocketMessageSide::FromWebsocket(message)),
            message = batch_updater_recv.recv() => message.ok().map(|message| WebsocketMessageSide::ToWebsocket(message)),
        };

        if let None = message {
            break;
        }

        let message = message.unwrap();

        match message {
            WebsocketMessageSide::ToWebsocket(message) => {
                // It sucks that axum requires binary websockets to send messages using
                // `Vec<u8>`, so we have to clone out of the `Arc`
                // It might be worth adding a semaphore before `message.to_vec()` to ensure we don't end up with
                // hundreds of copies of the vec at the same time
                ws.send(axum::extract::ws::Message::Binary(message.to_vec()))
                    .await
                    .unwrap();
            }
            WebsocketMessageSide::FromWebsocket(message) => {
                match message {
                    Ok(message) => match message {
                        // Ideally there'd be some rate-limiting on pings, but surely nobody would do anything
                        // bad on the internet :Clueless:
                        axum::extract::ws::Message::Ping(_) => (),
                        // We're not expecting to get any messages from users, so just close the connection
                        // if they send anything
                        _ => break,
                    },
                    Err(error) => {
                        trace!(?error, "error reading message from websocket");
                        break;
                    }
                }
            }
        }
    }

    ws.close().await.unwrap();
}

type SerializedOutgoingMessage = Vec<u8>;

async fn serialize_changes(
    mut incoming: mpsc::Receiver<LeaderboardsChanges>,
    outgoing: broadcast::Sender<Arc<SerializedOutgoingMessage>>,
) {
    while let Some(changes) = incoming.recv().await {
        let serialized = serde_json::to_vec(&OutgoingMessage::Changes { changes }).unwrap();
        outgoing.send(Arc::new(serialized)).unwrap();
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum OutgoingMessage {
    InitialLeaderboards {
        leaderboards: HashMap<LeaderboardName, LeaderboardElos>,
    },
    Changes {
        changes: LeaderboardsChanges,
    },
}

enum WebsocketMessageSide {
    ToWebsocket(Arc<SerializedOutgoingMessage>),
    FromWebsocket(Result<axum::extract::ws::Message, axum::Error>),
}

pub struct UnstartedWebsocketServer {
    ingest_send: mpsc::Sender<IngestedPerformance>,
    ingest_recv: mpsc::Receiver<IngestedPerformance>,
    leaderboards: HashMap<LeaderboardName, LeaderboardElos>,
}

impl UnstartedWebsocketServer {
    pub fn new(leaderboards: HashMap<LeaderboardName, LeaderboardElos>) -> Self {
        let (ingest_send, ingest_recv) = mpsc::channel(10_000);

        Self {
            ingest_send,
            ingest_recv,
            leaderboards,
        }
    }

    pub fn get_exporter_for_leaderboard(
        &self,
        leaderboard: LeaderboardName,
    ) -> ServerHandleExporter {
        if !self.leaderboards.contains_key(&leaderboard) {
            panic!("leaderboard {leaderboard:?} is not loaded");
            // TODO: this function should probably return a result or something instead of panicing
        }

        ServerHandleExporter {
            leaderboard,
            unbatched_send: self.ingest_send.clone(),
        }
    }

    pub async fn start(self) -> WebServerHandle {
        run_webserver(self.ingest_recv, self.leaderboards).await
    }
}

pub struct ServerHandleExporter {
    leaderboard: LeaderboardName,
    unbatched_send: mpsc::Sender<IngestedPerformance>,
}

impl Exporter for ServerHandleExporter {
    type Performance = PerformancePoints;
    type AuthorId = AuthorId;

    async fn export(&mut self, author_id: Self::AuthorId, performance: Self::Performance) {
        self.unbatched_send
            .send(IngestedPerformance {
                leaderboard_name: self.leaderboard.clone(),
                author_id,
                performance,
            })
            .await
            .unwrap();
    }
}
