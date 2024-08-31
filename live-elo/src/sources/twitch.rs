use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Message {
    text: String,
    // TODO: emote parsing, etc.
}

pub struct SourceConfig {
    channel_name: String,
    mpsc_capacity: usize,
}

pub struct SourceBuilder {
    channel_name: Option<String>,
    mpsc_capacity: Option<usize>,
}

impl SourceBuilder {
    pub fn new() -> Self {
        Self {
            channel_name: None,
            mpsc_capacity: None,
        }
    }

    pub fn channel_name(mut self, name: String) -> Self {
        self.channel_name = Some(name);
        self
    }

    pub fn mpsc_capacity(mut self, capacity: usize) -> Self {
        self.mpsc_capacity = Some(capacity);
        self
    }

    pub fn run(self) -> RunningSource {
        RunningSource::new(SourceConfig {
            channel_name: self.channel_name.unwrap(),
            mpsc_capacity: self.mpsc_capacity.unwrap(),
        })
    }
}

impl Default for SourceBuilder {
    fn default() -> Self {
        Self {
            channel_name: None,
            mpsc_capacity: Some(1_000),
        }
    }
}

pub struct RunningSource {
    task: tokio::task::JoinHandle<()>,
    mpsc_recv: tokio::sync::mpsc::Receiver<Message>,
}

impl RunningSource {
    fn new(config: SourceConfig) -> Self {
        let (send, recv) = tokio::sync::mpsc::channel(config.mpsc_capacity);
        let task = tokio::task::spawn(source_task(send, config.channel_name));

        Self {
            task,
            mpsc_recv: recv,
        }
    }

    pub fn get_abort_handle(&self) -> tokio::task::AbortHandle {
        self.task.abort_handle()
    }
}

impl lbo::MessageSource for RunningSource {
    type Message = super::Message;

    async fn next_message(&mut self) -> Option<Self::Message> {
        self
            .mpsc_recv
            .recv()
            .await
            .map(|msg| super::Message::Twitch(msg))
    }
}

impl Drop for RunningSource {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn source_task(send: tokio::sync::mpsc::Sender<Message>, channel_name: String) {
    // TODO: maybe pass login credentials here?
    let client_config = twitch_irc::ClientConfig::default();
    let (mut incoming_messages, client) =
        twitch_irc::TwitchIRCClient::<twitch_irc::SecureTCPTransport, _>::new(client_config);

    client.join(channel_name).unwrap();

    while let Some(irc_message) = incoming_messages.recv().await {
        if let twitch_irc::message::ServerMessage::Privmsg(message) = irc_message {
            send.send(Message {
                text: message.message_text,
            })
            .await
            .unwrap();
        }
    }
}
