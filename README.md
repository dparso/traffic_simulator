# Purpose
This project's primary purpose is to learn Rust. It's also to watch traffic patterns emerge and cars crash, because I sat in traffic one day wondering what it looked like at the macro level.

# TODOs
- remove DriverAgent from CarBundle, allow a user-driven car?
- react to window resize by adjusting a global WINDOW_WIDTH / _HEIGHT
- don't have to check for line of sight with cars not in your lane or adjacent lanes
    - avoid raycast, first check y position, if significantly behind, don't need to compare
- degrees of braking based on how close to front car?