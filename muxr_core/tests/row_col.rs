extern crate muxr;

mod row {
    use muxr::state::Row;

    #[test]
    fn div() {
        let x = Row::from(3);
        let y = x * 3u16;
        assert_eq!(y, Row::from(9));
    }

    #[test]
    fn mul() {
        let x = Row::from(20);
        let y = x / 5u16;
        assert_eq!(y, Row::from(4));
    }
}

mod col {
    use muxr::state::Col;

    #[test]
    fn div() {
        let x = Col::from(3);
        let y = x * 3u16;
        assert_eq!(y, Col::from(9));
    }

    #[test]
    fn mul() {
        let x = Col::from(20);
        let y = x / 5u16;
        assert_eq!(y, Col::from(4));
    }
}
