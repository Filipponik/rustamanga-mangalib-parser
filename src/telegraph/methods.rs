mod account;
mod page;

pub async fn create_account(
    short_name: &str,
    author_name: Option<&str>,
    author_url: Option<&str>
) {
    account::create(short_name, author_name, author_url).await
}

pub enum FieldToChange {
    ShortName,
    AuthorName,
    AuthorUrl,
}

pub async fn edit_account_info(
    access_token: &str,
    fields_to_change: Vec<FieldToChange>,
    short_name: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
) {
    account::edit(access_token, fields_to_change, short_name, author_name, author_url).await
}

pub async fn get_account_info(
    access_token: &str,
    fields: Vec<FieldToChange>
) {
    account::get(access_token, fields).await
}

pub async fn revoke_access_token(
    access_token: &str,
) {
    account::revoke_token(access_token).await
}