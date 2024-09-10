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
use tracing::{debug, info, instrument, trace};

use crate::sources::AuthorId;

// TODO: makes these into newtypes bruhge
pub type LeaderboardName = String;

pub type PerformancePoints = f32;
pub type Elo = f32;

pub type LeaderboardPerformances = HashMap<AuthorId, PerformancePoints>;
pub type LeaderboardElos = Vec<(AuthorId, Elo)>;

type LeaderboardPosition = usize;
type LeaderboardElosChanges = HashMap<LeaderboardPosition, (AuthorId, Elo)>;
type LeaderboardsChanges = HashMap<LeaderboardName, LeaderboardElosChanges>;

pub struct DummyEloProcessor {
    starting_leaderboard: LeaderboardElos,
}

impl DummyEloProcessor {
    pub fn new(starting_leaderboard: LeaderboardElos) -> Self {
        Self {
            starting_leaderboard,
        }
    }

    pub fn run(&self, performances: &LeaderboardPerformances) -> LeaderboardElos {
        // TODO: use a real elo processor please for the love of god
        // LeaderboardElos::new()
        performances
            .to_owned()
            .into_iter()
            .map(|(key, value)| (key, value))
            .collect()
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
    let elo_calculators: HashMap<LeaderboardName, DummyEloProcessor> = HashMap::from_iter(
        base_leaderboards
            .clone()
            .into_iter()
            .map(|(key, value)| (key, DummyEloProcessor::new(value))),
    );
    let mut all_current_performances: HashMap<LeaderboardName, LeaderboardPerformances> =
        HashMap::from_iter(
            base_leaderboards
                .keys()
                .map(|key| (key.to_owned(), LeaderboardPerformances::new())),
        );
    drop(base_leaderboards);

    while let Some(mut batch) = incoming.recv().await {
        trace!("trying to get write lock");
        let mut current_leaderboards = leaderboards.write().await;
        trace!(?batch, ?current_leaderboards, "got new batch");
        let before = current_leaderboards.clone();

        for (leaderboard_name, leaderboard) in current_leaderboards.iter_mut() {
            if let Some(changes) = batch.remove(leaderboard_name) {
                info!(?leaderboard_name, "ayayaya");
                let current_performances =
                    all_current_performances.get_mut(leaderboard_name).unwrap();
                for (author, delta) in changes {
                    // let score = current_performances.get_mut(&author).unwrap();
                    // *score += delta;
                    let score = current_performances.entry(author).or_default();
                    *score += delta;
                }
                trace!(?current_performances, "erm");

                let new_leaderboard = elo_calculators[leaderboard_name].run(current_performances);
                info!(?leaderboard_name, ?new_leaderboard, "updated leaderboard");
                *leaderboard = new_leaderboard;
            }
        }

        // info!(?current_performances, "updated performances for all leaderboards");
        info!(?before, ?current_leaderboards, "finding changes");
        let delta = find_changes(&before, &current_leaderboards);
        info!(?delta, "changes were");
        outgoing.send(delta).await.unwrap();
    }
}

type LeaderboardPerformancesDelta = HashMap<AuthorId, PerformancePoints>;
type IngestedPerformance = (LeaderboardName, AuthorId, PerformancePoints);
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

fn squash_batch(read: Vec<(String, AuthorId, f32)>) -> FullBatchedPerformances {
    let mut batch = FullBatchedPerformances::new();

    for (leaderboard, author, score) in read {
        let leaderboard = batch.entry(leaderboard).or_default();
        let author_score = leaderboard.entry(author).or_default();
        *author_score += score;
    }

    batch
}

async fn read_as_many_as_possible<T>(mpsc: &mut mpsc::Receiver<T>, into: &mut Vec<T>) {
    while let Some(value) = mpsc.recv().await {
        into.push(value);
    }
}

pub fn find_changes(
    from: &HashMap<LeaderboardName, LeaderboardElos>,
    to: &HashMap<LeaderboardName, LeaderboardElos>,
) -> LeaderboardsChanges {
    let mut changes = LeaderboardsChanges::new();
    for (name, before) in from {
        let leaderboard_changes = changes.entry(name.to_owned()).or_default();
        let now = to.get(name).unwrap();

        for (index, now_at) in now.into_iter().enumerate() {
            if before.get(index).map(|b| b != now_at).unwrap_or(true) {
                leaderboard_changes.insert(index, now_at.to_owned());
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

pub async fn run_webserver(
    ingest_recv: mpsc::Receiver<IngestedPerformance>,
    leaderboards: HashMap<LeaderboardName, LeaderboardElos>,
) -> WebServerHandle {
    let (serialized_send, serialized_recv) = broadcast::channel(10);
    let current_leaderboards = Arc::new(RwLock::new(leaderboards));
    let state = WebState {
        serialized_send: serialized_send.clone(),
        current_leaderboards: current_leaderboards.clone(),
    };

    let mut dependent_tasks = tokio::task::JoinSet::new();
    let (batched_send, batched_recv) = mpsc::channel(10_000);
    let (changes_send, changes_recv) = mpsc::channel(10_000);
    dependent_tasks.spawn(batch_performance_updates(ingest_recv, batched_send));
    dependent_tasks.spawn(turn_batched_performances_into_real_leaderboards(
        batched_recv,
        current_leaderboards.clone(),
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
    current_leaderboards: Arc<RwLock<HashMap<LeaderboardName, LeaderboardElos>>>,
}

async fn handle_websocket(mut ws: WebSocket, state: WebState) {
    // needs broadcast::Receiver<Arc<SerializedMessage>> for changes
    // ^^ fed from another task that serializes mpsc::Receiver<LeaderboardsChanges>
    // needs some kind of state to get current leaderboard for initial state
    // ^^ turn_batched_performances_into_real_leaderboards needs to share its current leaderboards somewhere
    //    ^^ Arc<RwLock<...>>?
    // must also read messages from websocket to keep alive

    let initial_state_lock = state.current_leaderboards.read().await;
    // This needs to happen whilst we have the read guard, as otherwise there's a risk
    // that we get a batch that has already been applied.
    // I don't think that would actually cause an issue come to think of it, but I'm unsure
    let mut batch_updater_recv = state.serialized_send.subscribe();
    let initial_state = initial_state_lock.clone();
    drop(initial_state_lock);
    let initial_state_serialized = serde_json::to_vec(&OutgoingMessage::InitialLeaderboards {
        leaderboards: initial_state,
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
pub enum OutgoingMessage {
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
}

impl UnstartedWebsocketServer {
    // TODO: this should probably have the leaderboards passed in here instead of
    //       in `start` so that `get_exporter_for_leaderboard` can do some error
    //       checking
    pub fn new() -> Self {
        let (ingest_send, ingest_recv) = mpsc::channel(10_000);

        Self {
            ingest_send,
            ingest_recv,
        }
    }

    // TODO: this should 100% have some kind of error checking to make sure
    //       that its actually referencing a leaderboard that exists
    pub fn get_exporter_for_leaderboard(
        &self,
        leaderboard: LeaderboardName,
    ) -> ServerHandleExporter {
        ServerHandleExporter {
            leaderboard,
            unbatched_send: self.ingest_send.clone(),
        }
    }

    pub async fn start(
        self,
        leaderboards: HashMap<LeaderboardName, LeaderboardElos>,
    ) -> WebServerHandle {
        run_webserver(self.ingest_recv, leaderboards).await
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
            .send((self.leaderboard.clone(), author_id, performance))
            .await
            .unwrap();
    }
}
