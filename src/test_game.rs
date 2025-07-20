use crate::game::{Game, PLAYER_COUNT};

#[test]
fn test_game_new() {
    let game = Game::new(true);
    assert_eq!(game.wall.tiles.len(), 136 - 13 * 4);
    for i in 0..PLAYER_COUNT {
        assert_eq!(game.hands[i].tiles.len(), 13);
    }
}

#[test]
fn test_draw_tile() {
    let mut game = Game::new(true);
    let initial_wall_len = game.wall.tiles.len();
    let tile = game.draw_tile();
    assert!(tile.is_some());
    assert_eq!(game.wall.tiles.len(), initial_wall_len - 1);
}

#[test]
fn test_discard_tile() {
    let mut game = Game::new(true);
    let initial_hand_len = game.hands[0].tiles.len();
    let initial_discard_len = game.discards[0].len();
    game.discard_tile(0, 0);
    assert_eq!(game.hands[0].tiles.len(), initial_hand_len - 1);
    assert_eq!(game.discards[0].len(), initial_discard_len + 1);
}
