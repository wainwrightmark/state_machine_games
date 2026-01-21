// use indexed_db_futures::database::Database;

// pub fn track_state<>(){

// }

// pub async fn set_up_db() -> indexed_db_futures::OpenDbResult<()> {
//     let db = Database::open("quiz_salad")
//     .with_version(1u8)
//     .with_on_upgrade_needed(|event,db|{
//         let r = db.create_object_store("quiz_salad_saves");
//     }).await;
// }