use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Subscriber {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub subscribed_at: String,
    pub status: String,
}
