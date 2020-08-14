extern crate apple_notes_rs_lib;
extern crate log;

fn main() {
    simple_logger::init().unwrap();
    /*let mut session = apple_imap::login();
    let folders = list_note_folders(&mut session);
    info!("Loaded {} folders", folders.len());
    folders.iter().for_each(|folder| {
        let uids = get_uids(&mut session, folder.to_string());
        uids.iter().for_each( |folder_uid_pairs| {
            let backup_folder = "Backup_Notes.".to_owned() + folder_uid_pairs.0 + "_backup";
            create_folder(&mut session, &backup_folder);
            folder_uid_pairs.1.iter().for_each( |id| {
                info!("{} {}", folder_uid_pairs.0, id.unwrap_or(0));
                if let Some(id) = id {
                    copy_uid(&mut session, id.to_string().as_ref(), &backup_folder);
                }
            })
        })
    })*/

}