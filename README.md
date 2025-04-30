# khzeb

## Roadmap

- [ ] Engine
    - [ ] Rendering
        - [ ] Post-processing filters
        - [ ] UI rendering
        - [ ] Batch individualistic objects
        - [ ] Batch drawing static tilemaps
    - [ ] Event-driven Debugger
        - [ ] TUI for tracing events at runtime
        - [ ] Good logging for events
        - [ ] State serialization/deserialization

## Renderer

The *renderer* is a component responsible for dispatching draw calls to the GPU. It is designed to be suitable for this specific.

The renderer is capable of drawing several types of primitives:

1. **Tilemap**. Dense quads with textures sampled from a texture atlas.
2. **Batch**. Sparse quads with specific positions.
3. **Particle**. Massive quantity of volatile textured quads with generic rules for movement and trajectory.
4. **Texture**. A texture drawn from a specific buffer. The buffer is CPU-side and is dispatched the GPU regularly. Any drawing to the texture is per-pixel.
5. **Overlay**. Overlay of the screen texture for screen-space effects.

Different parts of each can be controlled. S is for specific, so the property is controlled specifically for each instance, G is group, so the property is controlled for the entire group, N is none, so the property is preset for that type of primitive. The table below is not accurate, but still possible.

| Primitive | Texture | Color | Position | Shape | Amount |
| --- | --- | --- | --- | --- | --- |
| Particle | G | G | G | N | Dynamic, Limited on creation |
| Tilemap | G | S | G | N | Fixed, Selected on creation | 
| Batch | G | S | S | N | Dynamic, Limited on creation |
| Texture | S | S | S | N | Fixed, 1 |
| Overlay | N | S | N | N | Fixed, 1 |
