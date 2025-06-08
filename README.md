# juniper-graphql-worker

High-performance GraphQL APIs powered by Rust and Cloudflare Workers.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

juniper-graphql-worker combines the speed and safety of Rust with the global edge network of Cloudflare Workers to deliver a lightning-fast GraphQL API. This project uses the Juniper GraphQL library for Rust, Axum for HTTP routing, and the Cloudflare Workers platform for serverless deployment.

## Features

- **Serverless**
- **Rust Performance**
- **GraphQL API**
- **Interactive Tools**
- **Type Safety**
- **Modular Design**
- **WebSocket Subscriptions** - Real-time updates via GraphQL subscriptions

## Getting Started

> [Bun](https://bun.sh) manages the wrangler dependency and provides task execution. Source code for this project is written in Rust and targets `wasm32-unknown-unknown`.    

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.86.0+)
- [Bun](https://bun.sh) (drop-in replacement for npm/yarn/pnpm)
- [Wrangler](https://developers.cloudflare.com/workers/wrangler/install-and-update/) (Cloudflare Workers CLI)

### Installation

1. Clone the repository
   ```bash
   git clone https://github.com/geoffsee/juniper-graphql-worker.git
   cd juniper-graphql-worker
   ```

2. Install dependencies
   ```bash
   bun i
   ```

3. Start the development server
   ```bash
   bun dev
   ```

4. Navigate to `http://localhost:3000/playground` to interact with the GraphQL API
   - Queries and mutations are available at `http://localhost:3000/graphql`
   - Subscriptions are available at `http://localhost:3000/subscriptions`

> **Optional**: Deploy this api to the internet `bunx wrangler deploy`

## Project Structure

- `src/`
    - `lib.rs` - Main application entry point and server setup
    - `context.rs` - GraphQL context setup
    - `database.rs` - Database connection and operations
    - `models.rs` - GraphQL object and input type definitions
    - `schema.rs` - GraphQL schema with queries, mutations, and subscriptions

## API Example

This GraphQL API implements a Star Wars-themed data model. Queries and mutations are available at the `/graphql` endpoint, and subscriptions are available at the `/subscriptions` endpoint.

Here's an example query:

```graphql
query {
  human(id: "1") {
    id
    name
    homePlanet
    appearsIn
  }
}
```

And an example mutation:

```graphql
mutation {
  createHuman(
    newHuman: {
      name: "Han Solo",
      homePlanet: "Corellia",
      appearsIn: [NEWHOPE, EMPIRE, JEDI]
    }
  ) {
    id
    name
    homePlanet
  }
}
```

And an example subscription:

```graphql
subscription {
  humanCreated {
    id
    name
    homePlanet
    appearsIn
  }
}
```

Note: The actual field name in the schema is `human_created`, but GraphQL converts it to camelCase in the client.

The subscription will receive updates whenever a new human is created using the `createHuman` mutation.

## Deployment

Deploy to Cloudflare Workers:

```bash
bunx wrangler deploy
```

## Development

### Extending the API

To extend the schema with new models:

1. Add model definitions in `src/models.rs`
2. Add new queries and mutations in `src/schema.rs`

> Note: Your approach will vary based on your requirements. 


## Performance

Rust's efficiency combined with Cloudflare's edge network provides:

- Low latency responses
- Minimal cold starts
- Global distribution
- Small binary size

## Tech Stack

- **Rust** - Systems programming language
- **Juniper** - GraphQL server library for Rust
- **Juniper Subscriptions** - GraphQL subscriptions support for Juniper
- **Juniper GraphQL WS** - WebSocket transport for GraphQL subscriptions
- **Axum** - Web framework for Rust with WebSocket support
- **Cloudflare Workers** - Serverless edge computing platform
- **Wrangler** - CLI tool for Cloudflare Workers

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License

## Acknowledgments

- [Juniper](https://github.com/graphql-rust/juniper) - GraphQL server library for Rust
- [Juniper Subscriptions](https://github.com/graphql-rust/juniper/tree/master/juniper_subscriptions) - GraphQL subscriptions support for Juniper
- [Juniper GraphQL WS](https://github.com/graphql-rust/juniper/tree/master/juniper_graphql_ws) - WebSocket transport for GraphQL subscriptions
- [Axum](https://github.com/tokio-rs/axum) - Web application framework for Rust
- [Cloudflare Workers](https://workers.cloudflare.com/) - Serverless platform

---

Created by [Geoff Seemueller](https://github.com/geoffsee)
