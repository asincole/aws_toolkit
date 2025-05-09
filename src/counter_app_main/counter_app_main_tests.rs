// use super::*;
// use ratatui::style::Style;
// 
// #[test]
// fn render() {
//     let app = App::default();
//     let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
// 
//     app.render(buf.area, &mut buf);
// 
//     let mut expected = Buffer::with_lines(vec![
//         "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
//         "┃                    Value: 0                    ┃",
//         "┃                                                ┃",
//         "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
//     ]);
//     let title_style = Style::new().bold();
//     let counter_style = Style::new().yellow();
//     let key_style = Style::new().blue().bold();
//     expected.set_style(Rect::new(14, 0, 22, 1), title_style);
//     expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
//     expected.set_style(Rect::new(13, 3, 6, 1), key_style);
//     expected.set_style(Rect::new(30, 3, 7, 1), key_style);
//     expected.set_style(Rect::new(43, 3, 4, 1), key_style);
// 
//     assert_eq!(buf, expected);
// }
// 
// #[test]
// fn handle_key_event() -> Result<()> {
//     let mut app = App::default();
//     app.handle_key_event(KeyCode::Right.into()).unwrap();
//     assert_eq!(app.counter, 1);
// 
//     app.handle_key_event(KeyCode::Left.into()).unwrap();
//     assert_eq!(app.counter, 0);
// 
//     let mut app = App::default();
//     app.handle_key_event(KeyCode::Char('q').into()).unwrap();
//     assert!(app.exit);
// 
//     Ok(())
// }
// 
// #[test]
// #[should_panic(expected = "attempt to subtract with overflow")]
// fn handle_key_event_panic() {
//     let mut app = App::default();
//     let _ = app.handle_key_event(KeyCode::Left.into());
// }
// 
// #[test]
// fn handle_key_event_overflow() {
//     let mut app = App::default();
//     assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
//     assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
//     assert_eq!(
//         app.handle_key_event(KeyCode::Right.into())
//             .unwrap_err()
//             .to_string(),
//         "counter overflow"
//     );
// }
