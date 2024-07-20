use crate::telegraph::methods::FieldToChange;

pub async fn create(
    short_name: &str,
    author_name: Option<&str>,
    author_url: Option<&str>
) {
    todo!();
}

pub async fn edit(
    access_token: &str,
    fields_to_change: Vec<FieldToChange>,
    short_name: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
) {
    todo!();
}

pub async fn get(
    access_token: &str,
    fields: Vec<FieldToChange>
) {
    todo!();
}

pub async fn revoke_token(
    access_token: &str,
) {
    todo!();
}
