# 🛸 Space Shooter (Rust)

¡Un shoot'em up espacial para la terminal escrito en Rust, rápido y con audio real! 🦀✨

## ✨ Características

- ⚡ Render por **framebuffer** propio para una animación fluida en la terminal.
- 🌠 Campo de estrellas con parallax (starfield).
- 👾 Oleadas de enemigos, balas del jugador y enemigas, y sprites en ASCII.
- 🔊 Audio real con **rodio** (efectos de sonido y música).
- 🎮 Modo extra `--rayman`: una demo alternativa incluida.
- 🦀 Escrito 100% en Rust con `crossterm` para el control de terminal.

## 🚀 Cómo jugar / ejecutar

```bash
cargo run --release
```

Modo demo alternativo:

```bash
cargo run --release -- --rayman
```

> Necesitas el toolchain de **Rust** (`cargo`). La carpeta `target/` se genera al compilar.

## 🎮 Controles

- **Flechas / WASD**: mover la nave.
- **Espacio**: disparar.
- **Q / Esc**: salir.

## 🛠️ Tecnología

- **Rust** (edición 2021).
- [`crossterm`](https://crates.io/crates/crossterm) — control de terminal.
- [`rodio`](https://crates.io/crates/rodio) — audio.
- [`rand`](https://crates.io/crates/rand) — aleatoriedad.

## 📦 Parte de mi colección de juegos

El experimento en Rust de mi colección de juegos de terminal. 🕹️
