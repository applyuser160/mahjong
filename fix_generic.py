import re

with open('src/mahjong/tile.rs', 'r') as f:
    content = f.read()

content = content.replace('pub fn from_usize(n: usize) -> TileName {\n        let n = n as usize;', 'pub fn from_usize(n: usize) -> TileName {')

with open('src/mahjong/tile.rs', 'w') as f:
    f.write(content)
