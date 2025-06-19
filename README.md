# Adrastos

A killer Backend-as-a-Service (BaaS) written in Rust.

## What is it

This is my attempt at making a BaaS like Supabase, Appwrite and Pocketbase and combining the best of all of these products. It includes a database with configurable and custom schemas, auth including OAuth, MFA, emailing and passkeys and project management. This is all wrapped up in a single rust binary which includes a dashboard too.

The main entrypoint for the code is `crates/app/src/main.rs`.
