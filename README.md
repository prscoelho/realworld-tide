# The authentication routes of the realworld spec api in Rust with Tide-rs

[API Spec](https://github.com/gothinkster/realworld/tree/master/api)

[Tide](https://github.com/http-rs/tide)

# Routes

https://github.com/gothinkster/realworld/tree/master/api#authentication

Login

`POST /api/users/login`

Register

`POST /api/users`

Get current user (requires login token)

`GET /api/user`

Update user (requires login token)

`PUT /api/user`

# Running

Since our sql statements are being verified at compile time, the database must be running before we compile the project.

## Start the database

`docker-compose up -d`

## Running

With the database up:

`cargo run --release`

## Testing routes

Endpoint is served at localhost:8080

### Register

```bash
curl --header "Content-Type: application/json" \
     --request POST \
     --data '{"user":{"username":"hello","email":"hello@example.com","password":"password"}}' \
     http://localhost:8080/api/users
```

### Verify login

The previous register response should contain a token field, replace $token with it:

```bash
curl --header "Content-Type: application/json" \
     --header "Authorization: Token $token" \
     --request GET \
     http://localhost:8080/api/user
```

### Login
```bash
curl --header "Content-Type: application/json" \
     --request POST \
     --data '{"user":{"email":"hello@example.com","password":"password"}}' \
     http://localhost:8080/api/users/login
```

## Cleaning up

To stop and delete the db container and volume:

`docker-compose down`
