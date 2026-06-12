use indexed_db_futures::{Build, BuildSerde, database::Database, prelude::QuerySource};
use serde::{Serialize, de::DeserializeOwned};

pub struct DatabaseHandler<Command: DatabaseCommand> {
    pub db: Database,
    pub output_sender: std::sync::mpsc::Sender<Command::Output>,
    pub command_receiver: async_channel::Receiver<Command>,
}

pub trait DatabaseCommand {
    type Output;

    #[allow(async_fn_in_trait)]
    async fn run_command(&self, db: &Database) -> Result<Option<Self::Output>, anyhow::Error>;
}

impl<TCommand: DatabaseCommand> DatabaseHandler<TCommand> {
    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        while let Ok(command) = self.command_receiver.recv().await {
            leptos::logging::log!("Received Command");
            match command.run_command(&self.db).await {
                Ok(Some(output)) => match self.output_sender.send(output) {
                    Ok(()) => {}
                    Err(err) => {
                        anyhow::bail!("{}", err);
                    }
                },
                Ok(None) => {}
                Err(err) => {
                    anyhow::bail!("{}", err);
                }
            }
        }

        return Ok(());
    }
}

pub async fn try_write_object<T: Serialize>(
    t: T,
    db: &Database,
    store: &'static str,
) -> Result<(), anyhow::Error> {
    let transaction = db
        .transaction(store)
        .with_mode(web_sys::IdbTransactionMode::Readwrite)
        .build()
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    let store = transaction
        .object_store(store)
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    store
        .put::<T>(t)
        .serde()
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    transaction
        .commit()
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    Ok(())
}

pub async fn try_read_object<T: DeserializeOwned>(
    db: &Database,
    store: &'static str,
    key: &str,
) -> Result<Option<T>, anyhow::Error> {
    let transaction = db
        .transaction(store)
        .with_mode(web_sys::IdbTransactionMode::Readonly)
        .build()
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let store = transaction
        .object_store(store)
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let get_result: Option<T> = store
        .get(key)
        .serde()
        .map_err(|err| anyhow::anyhow!("{err}"))?
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    transaction
        .commit()
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    Ok(get_result)
}
