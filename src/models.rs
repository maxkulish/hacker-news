use super::schema::{posts, users, comments};
use crate::PostForm;
use diesel::{Insertable, Queryable, Identifiable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    pub title: String,
    pub link: String,
    pub author: i32,
    pub created_at: chrono::NaiveDateTime,
}

impl NewPost {
    pub fn from_post_form(form: PostForm, uid: i32) -> Self {
        NewPost {
            title: form.title,
            link: form.link,
            author: uid,
            created_at: chrono::Local::now().naive_utc(),
        }
    }
}

#[derive(Serialize, Debug, Queryable, Identifiable)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub link: String,
    pub author: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Post)]
pub struct Comment {
    pub id: i32,
    pub comment: String,
    pub post_id: i32,
    pub user_id: i32,
    pub parent_comment_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Insertable)]
#[table_name = "comments"]
pub struct NewComment {
    pub comment: String,
    pub post_id: i32,
    pub user_id: i32,
    pub parent_comment_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

impl NewComment {
    pub fn new(comment: String, post_id: i32, user_id: i32, parent_comment_id: Option<i32>) -> Self {
        NewComment {
            comment,
            post_id,
            user_id,
            parent_comment_id,
            created_at: chrono::Local::now().naive_utc(),
        }
    }
}