//Make sure to mod this as first item in lib.rs

/// Small macro that sets the metadata_uuid foreign key
/// for all provided bodies
macro_rules! note {
    ($metadata:expr $(, $body:expr)+) => {
        {
            let uuid = $metadata.uuid.clone();
            let mut temp_set = Vec::new();
            $(
                let mut mutable_body = $body;
                mutable_body.metadata_uuid = uuid.clone();
                temp_set.push(mutable_body);
            )*
            ($metadata,temp_set)
        }
    };
}

macro_rules! set {
    ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
        {
            let mut temp_set = HashSet::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
}