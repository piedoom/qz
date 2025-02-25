- new enum that has handle or asset id
- always serialize to asset id
- create custom asset loader that converts between handle type on deserialize (with dependencies?)

# qz

## Program structure

1. `main.rs` creates a new app with `ClientPlugins` (our entire game)
2. `plugins/mod.rs` contains `ClientPlugins` and registers all plugins in the game
3. `plugins/world.rs` has much of the world spawning logic. See its setup for the game world entrypoint

Commands are used extensively to build the world. With `crate::prelude::*` included, look under the scope of `trigger::` to find possible commands.

### World generation

1. A graph with a depth of x to y is created
2. Upon loading each node, the world is generated or loaded from file
3. The world consists of serialized commands to build the scene

## Features

- Input management ready to go with `leafwing-input-manager`
- `ron` and 2d/3d assets ready to load
- Automatic window resizing on wasm
- Loading states courtesy of `bevy_asset_loader`
- Settings `ron` set up and ready for application use
- wasm friendly
- Scalable file structure

You'll also probably want a physics library (`bevy_xpbd_2d/3d`) and UI (`egui`),
but those aren't included since they're not always a requirement of every game,
and there alternatives to egui.

## Hot reloading

To run with asset hot reloading, use

```
cargo run --features bevy/file_watcher
```

## Structure

- components
- plugins (private system groups)
- resources (and assets)
- states
- util (+public systems)

Couple of important points:

- The project uses its own `prelude`. Our game types are so commonly needed that
  the pre-made folders are publicly exported in a `prelude` mod. You should use
  this often, because it saves a lot of trouble (`use crate::prelude::*`).
- I postfix the type of thing that something is, _except_ for components, e.g.
  "GameAsset", "MyResource", "AppState". Components do not follow this pattern
  because they are so frequently needed, and can be made obvious in systems
  without the additional keyword. I find Resources and States are slightly harder
  to intuit, even though they derive certain things. Idk. Theres some other
  reasons I can't articulate.
- Most of the game logic resides in `plugins`. Most systems are meant to run in
  groups of some kind, so they're actually private functions within plugin
  files. The only thing that is exposed is the `Plugin`, which is then consumed by
  a `PluginGroup`. Those groups are by default `CorePlugins` and `ClientPlugins`.
  This is to make separation easier for, e.g. servers. (Imagine a
  client app that uses `ClientPlugins` and a dedicated server app that just uses
  `CorePlugins`). If you actually want to reuse a system across different plugins,
  it probably should go in `util` as a helper system.
- All components go in `components`, yeah. Resources/assets in `resources`. States
  in `states`.
