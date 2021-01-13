
pub trait MergeableNoteBody {
    fn needs_local_merge(&self) -> bool;
    fn get_message_id(&self) -> Option<String>;
    fn all_message_ids(&self) -> Vec<String>;
}