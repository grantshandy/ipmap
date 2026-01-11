# `ipgeo` Tauri Plugin

This is much of the application state/functionality bridged from the backend to the frontend
as encapsulated in a single crate/npm package for type safety.

This is a TypeScript singleton class with Svelte runes state that reflects the state
of the Rust IP-geolocation database backed by immutable memory-mapped files. It encapsulates
the state and lifetimes of the resources on both the frontend and backend in the `Database` typescript class, or `DbState` Rust struct.
