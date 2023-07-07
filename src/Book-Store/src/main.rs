use actix_web::{get, post, put, delete, web, App, HttpResponse, HttpServer, Responder};
use actix_web::error::ErrorInternalServerError;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};
use mongodb::bson;
use serde::{Serialize, Deserialize};
use futures::stream::TryStreamExt;
use futures::StreamExt;


async fn get_mongo_client() -> Result<Client, mongodb::error::Error> {
    let client_options = ClientOptions::parse(
        "mongodb://mongodb+srv://jkfp:221997@jane.jxr0n15.mongodb.net/?retryWrites=true&w=majority"
    ).await?;
    let client = Client::with_options(client_options)?;

    Ok(client)
}

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    id: u32,
    title: String,
    author: String,
}

#[get("/books")]
async fn get_books() -> Result<HttpResponse, actix_web::Error> {
    let client = get_mongo_client().await.map_err(ErrorInternalServerError)?;
    let db = client.database("try-from-rust");
    let collection = db.collection::<Document>("books");
    let mut cursor = collection.find(None, None).await.map_err(ErrorInternalServerError)?;
    let mut books: Vec<Book> = Vec::new();

    while let Some(result) = cursor.next().await {
        let doc = result.map_err(ErrorInternalServerError)?;
        let book: Book = bson::from_document(doc).map_err(ErrorInternalServerError)?;
        books.push(book);
    }

    Ok(HttpResponse::Ok().json(books))
}


#[get("/books/{id}")]
async fn get_book(id: web::Path<u32>) -> impl Responder {
    let book_id = id.into_inner(); // Mengambil nilai ID buku dari web::Path

    let client = get_mongo_client().await.unwrap();
    let db = client.database("try-from-rust");
    let collection = db.collection("books");

    let filter = doc! { "id": book_id };
    if let Ok(Some(doc)) = collection.find_one(filter, None).await {
        let book: Book = bson::from_document(doc).unwrap();
        HttpResponse::Ok().json(book)
    } else {
        HttpResponse::NotFound().body("Book not found")
    }
}

#[post("/books")]
async fn create_book(book: web::Json<Book>) -> impl Responder {
    let client = get_mongo_client().await.unwrap();
    let db = client.database("try-from-rust");
    let collection = db.collection("books");

    let new_book = Book {
        id: book.id,
        title: book.title.clone(),
        author: book.author.clone(),
    };

    let doc = bson::to_document(&new_book).unwrap();
    collection.insert_one(doc, None).await.unwrap();

    HttpResponse::Created().json(new_book)
}

#[put("/books/{id}")]
async fn update_book(web::Path(id): web::Path<u32>, book: web::Json<Book>) -> impl Responder {
    let book_id = id.into_inner();

    let client = get_mongo_client().await.unwrap();
    let db = client.database("try-from-rust");
    let collection: mongodb::Collection<Book> = db.collection("books");

    let filter = doc! { "id": book_id };
    let update = doc! { "$set": { "title": &book.title, "author": &book.author } };

    if let Ok(Some(_)) = collection.find_one_and_update(filter, update, None).await {
        let updated_book = Book {
            id: book_id,
            title: book.title.clone(),
            author: book.author.clone(),
        };
        HttpResponse::Ok().json(updated_book)
    } else {
        HttpResponse::NotFound().body("Book not found")
    }
}


#[delete("/books/{id}")]
async fn delete_book((id, book): web::Path<u32>) -> impl Responder {
    let client = get_mongo_client().await.unwrap();
    let db = client.database("try-from-rust");
    let collection = db.collection("books");

    let filter = doc! { "id": id };
    if let Ok(Some(_)) = collection.find_one_and_delete(filter, None).await {
        HttpResponse::NoContent()
    } else {
        HttpResponse::NotFound().body("Book not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_books)
            .service(get_book)
            .service(create_book)
            .service(update_book)
            .service(delete_book)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
