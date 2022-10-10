//Make sure to mod this as first item in lib.rs

/// Small macro that sets the metadata_uuid foreign key
/// for all provided bodies
#[macro_export]
macro_rules! note {
    ($metadata:expr $(, $body:expr)+) => {
        {
            let localnote = $metadata;
            let uuid = localnote.uuid.clone();
            let mut temp_set = Vec::new();
            $(
                let mut mutable_body = $body;
                mutable_body.metadata_uuid = uuid.clone();
                temp_set.push(mutable_body);
            )*
            crate::notes::localnote::LocalNote {
                metadata: localnote,
                body: temp_set
            }
        }
    };
}

#[cfg(test)]
macro_rules! set {
    ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
        {
            #[allow(unused_mut)]
            let mut temp_set = HashSet::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
}

#[cfg(test)]
mod macro_test {
    use crate::builder::{NotesMetadataBuilder, BodyMetadataBuilder};
    use crate::notes::localnote::LocalNote;

    #[test]
    fn note_macro_uuid() {
        let metadata = NotesMetadataBuilder::new().build();
        let body = BodyMetadataBuilder::new().build();

        let note: LocalNote = note!(
            metadata.clone(),
            body.clone()
    );

        let note2: LocalNote = note!(
            NotesMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build()
    );

        info!("{}", note2.metadata.uuid);
        assert_eq!(note.metadata.uuid, note.body.first().unwrap().metadata_uuid);
    }
}

