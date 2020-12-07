#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use models::*;
use serde::Deserialize;
use tera::{Context, Tera};

#[derive(Deserialize)]
pub struct PostForm {
    title: String,
    link: String,
}

#[derive(Debug, Deserialize)]
struct Submission {
    title: String,
    link: String,
}

async fn login(tera: web::Data<Tera>, ident: Identity) -> impl Responder {
    let mut data = Context::new();
    data.insert("title", "Login");

    if let Some(_) = ident.identity() {
        return HttpResponse::Ok().body("Already logged in");
    }

    let rendered = tera.render("login.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn process_login(data: web::Form<LoginUser>, ident: Identity) -> impl Responder {
    use schema::users::dsl::{username, users};

    let connection = establish_connection();
    let user = users
        .filter(username.eq(&data.username))
        .first::<User>(&connection);

    match user {
        Ok(u) => {
            if u.password == data.password {
                let session_token = String::from(u.username);
                ident.remember(session_token);
                println!("{:?}", data);
                HttpResponse::Ok().body(format!("Logged in: {}", data.username))
            } else {
                HttpResponse::Ok().body("Password is incorrect.")
            }
        }
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::Ok().body("User doesn't exists")
        }
    }
}

async fn logout(ident: Identity) -> impl Responder {
    ident.forget();
    HttpResponse::Ok().body("Logged out")
}

async fn submission(tera: web::Data<Tera>, ident: Identity) -> impl Responder {
    let mut data = Context::new();
    data.insert("title", "Submit a Post");

    if let Some(_) = ident.identity() {
        let rendered = tera.render("submission.html", &data).unwrap();
        return HttpResponse::Ok().body(rendered);
    }

    HttpResponse::Unauthorized().body("User not logged in")
}

async fn process_submission(data: web::Form<PostForm>, ident: Identity) -> impl Responder {
    if let Some(id) = ident.identity() {
        use schema::users::dsl::{username, users};

        let connection = establish_connection();
        let user: Result<User, diesel::result::Error> =
            users.filter(username.eq(id)).first(&connection);

        return match user {
            Ok(u) => {
                let new_post = NewPost::from_post_form(data.into_inner(), u.id);

                use schema::posts;

                diesel::insert_into(posts::table)
                    .values(&new_post)
                    .get_result::<Post>(&connection)
                    .expect("Error saving post");

                HttpResponse::Ok().body("Submitted")
            }
            Err(e) => {
                println!("{:?}", e);
                HttpResponse::Ok().body("Failed to find user")
            }
        };
    }

    HttpResponse::Unauthorized().body("User not logged in")
}

async fn signup(tera: web::Data<Tera>) -> impl Responder {
    let mut data = Context::new();
    data.insert("title", "Sign Up");

    let rendered = tera.render("signup.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn process_signup(data: web::Form<NewUser>) -> impl Responder {
    use schema::users::dsl::*;

    let connection = establish_connection();

    diesel::insert_into(users)
        .values(&*data)
        .get_result::<User>(&connection)
        .expect("Error registering user");

    println!("{:?}", data);
    HttpResponse::Ok().body(format!("Successfully saved user: {}", data.username))
}

async fn index(tera: web::Data<Tera>) -> impl Responder {
    use schema::posts::dsl::posts;
    use schema::users::dsl::users;

    let connection = establish_connection();
    let all_posts: Vec<(Post, User)> = posts
        .inner_join(users)
        .load(&connection)
        .expect("Error retrieving all posts");

    let mut data = Context::new();
    data.insert("title", "Hacker Clone");
    data.insert("posts_users", &all_posts);

    let rendered = tera.render("index.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

async fn post_page(
    tera: web::Data<Tera>,
    ident: Identity,
    web::Path(post_id): web::Path<i32>,
) -> impl Responder {
    use schema::posts::dsl::{posts};
    use schema::users::dsl::{users};

    let connection = establish_connection();

    let post: Post = posts.find(post_id)
        .get_result(&connection)
        .expect("Failed to find posts");

    let user: User = users.find(post.author)
        .get_result(&connection)
        .expect("Failed to find user");

    let comments: Vec<(Comment, User)> = Comment::belonging_to(&post)
        .inner_join(users)
        .load(&connection)
        .expect("Failed to find comments");

    let mut data = Context::new();
    data.insert("title", &format!("{} - HackerClone", post.title));
    data.insert("post", &post);
    data.insert("user", &user);
    data.insert("comments", &comments);

    if let Some(_) = ident.identity() {
        data.insert("logged in", "true");
    } else {
        data.insert("logged in", "false");
    }

    let rendered = tera.render("post.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[derive(Deserialize)]
struct CommentForm {
    comment: String,
}

async fn comment(
    data: web::Form<CommentForm>,
    ident: Identity,
    web::Path(post_id): web::Path<i32>
) -> impl Responder {

    if let Some(id) = ident.identity() {
        use schema::posts::dsl::{posts};
        use schema::users::dsl::{users, username};

        let connection = establish_connection();

        let post: Post = posts.find(post_id)
            .get_result(&connection)
            .expect("Failed to find post");

        let user: Result<User, diesel::result::Error> = users
            .filter(username.eq(id))
            .first(&connection);

        return match user {
            Ok(u) => {
                let parent_id = None;
                let new_comment = NewComment::new(
                    data.comment.clone(),
                    post.id,
                    u.id,
                    parent_id
                );

                use schema::comments;
                diesel::insert_into(comments::table)
                    .values(&new_comment)
                    .get_result::<Comment>(&connection)
                    .expect("Error saving comment");

                HttpResponse::Ok().body("Commented")
            }
            Err(e) => {
                println!("{:?}", e);
                HttpResponse::Ok().body("User not found")
            }
        }
    }

    HttpResponse::Unauthorized().body("Not logged in")
}

async fn user_profile(tera: web::Data<Tera>, web::Path(requested_user): web::Path<String>) -> impl Responder {
    use schema::users::dsl::{username, users};

    let connection = establish_connection();
    let user: User = users.filter(username.eq(requested_user))
        .get_result(&connection)
        .expect("Failed to find user");

    let posts: Vec<Post> = Post::belonging_to(&user)
        .load(&connection)
        .expect("Failed to find posts");

    let comments: Vec<Comment> = Comment::belonging_to(&user)
        .load(&connection)
        .expect("Failed to find comments");

    let mut data = Context::new();
    data.insert("title", &format!("{} - Profile", user.username));
    data.insert("user", &user);
    data.insert("posts", &posts);
    data.insert("comments", &comments);

    let renderer = tera.render("profile.html", &data).unwrap();
    HttpResponse::Ok().body(renderer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let tera = Tera::new("templates/**/*").unwrap();
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .secure(false),
            ))
            .data(tera)
            .route("/", web::get().to(index))
            .route("/signup", web::get().to(signup))
            .route("/signup", web::post().to(process_signup))
            .route("/login", web::get().to(login))
            .route("/login", web::post().to(process_login))
            .route("/logout", web::to(logout))
            .route("/submission", web::get().to(submission))
            .route("/submission", web::post().to(process_submission))
            .service(
                web::resource("/post/{post_id}")
                    .route(web::get().to(post_page))
                    .route(web::post().to(comment)))
            .service(
                web::resource("/user/{username}")
                    .route(web::get().to(user_profile))
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
