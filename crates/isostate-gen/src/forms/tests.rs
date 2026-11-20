use super::*;

fn ask(year: i32, storeys: u8, lot: Lot, use_: Use, pick: f32) -> Ask {
    Ask { knows: Knows::by_year(year), year, storeys, lot, use_, pick }
}

#[test]
fn the_catalogue_is_whole_and_in_order() {
    for (i, s) in FORMS.iter().enumerate() {
        assert!(FORMS[..i].iter().all(|o| o.shape != s.shape), "{:?} has two lines", s.shape);
        assert!(s.years.0 <= s.years.1 && s.storeys.0 <= s.storeys.1 && s.storeys.0 >= 1, "{}", s.name);
        assert!(!s.needs.is_empty() && s.ground.0 > 0.0 && s.ground.1 > 0.0, "{}", s.name);
        assert_eq!(spec(s.shape).name, s.name);
    }
}

/// The pass the brief asks for: every year from 1750 to 2050 and every height
/// from one to forty storeys, on every kind of lot and for every use, gets an
/// answer, and the same answer each time it is asked.
#[test]
fn every_question_gets_an_answer_and_always_the_same() {
    let mut asked = 0;
    for year in 1750..=2050 {
        for storeys in 1..=40u8 {
            for lot in Lot::ALL {
                for use_ in Use::ALL {
                    for pick in [0.0, 0.37, 0.99] {
                        let a = ask(year, storeys, lot, use_, pick);
                        let x = choose(&a);
                        assert_eq!(x, choose(&a), "{a:?}");
                        let s = spec(x.shape);
                        assert!(x.storeys >= s.storeys.0 && x.storeys <= s.storeys.1, "{a:?}: {x:?}");
                        // what is chosen is what the town knows how to build
                        assert!(a.knows.all(s.needs), "{a:?}: {} needs what the town does not know", s.name);
                        asked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(asked, 301 * 40 * 4 * 3 * 3);
}

/// Every form answers some question exactly: none is dead weight.
#[test]
fn every_form_is_the_answer_to_some_question() {
    for s in FORMS {
        let year = s.needs.iter().map(|k| k.year()).fold(s.years.0, i32::max);
        assert!(year <= s.years.1, "{} is never known in its own years", s.name);
        let found = (0..40).any(|i| {
            let x = choose(&ask(year, s.storeys.0, s.lot, s.use_, i as f32 / 40.0));
            x.fits && x.shape == s.shape
        });
        assert!(found, "{} answers no question", s.name);
    }
}

#[test]
fn the_answers_follow_technology_year_and_height() {
    let shape = |year: i32, storeys: u8, lot: Lot, use_: Use| choose(&ask(year, storeys, lot, use_, 0.0)).shape;
    assert_eq!(shape(1800, 1, Lot::House, Use::Dwellings), Shape::Cottage);
    assert_eq!(shape(1800, 3, Lot::House, Use::Dwellings), Shape::Townhouse);
    assert_eq!(shape(1930, 2, Lot::Joined, Use::Dwellings), Shape::SemisInterwar);
    assert_eq!(shape(1965, 10, Lot::Block, Use::Dwellings), Shape::Slab);
    assert_eq!(shape(1965, 16, Lot::Joined, Use::Dwellings), Shape::PointBlock);
    assert_eq!(shape(2030, 30, Lot::Joined, Use::Dwellings), Shape::FlatsTower);
    assert_eq!(shape(1910, 5, Lot::Block, Use::Dwellings), Shape::Court1905);
    assert_eq!(shape(1935, 20, Lot::Joined, Use::Offices), Shape::SteppedTower);
    assert_eq!(shape(2030, 30, Lot::Joined, Use::Offices), Shape::GlassTower);
    assert_eq!(shape(1980, 2, Lot::Block, Use::Sheds), Shape::RetailShed);
    // a tall question asked before anyone knows how to build tall gets the tallest that can be built
    let early = choose(&ask(1850, 20, Lot::Joined, Use::Offices, 0.0));
    assert!(!early.fits && spec(early.shape).needs.iter().all(|k| k.year() <= 1850), "{early:?}");
    // a town that has not found the steel frame by 1935 builds no stepped tower
    let a = Ask { knows: Knows::by_year(1935).without(Know::SteelFrame), ..ask(1935, 20, Lot::Joined, Use::Offices, 0.0) };
    assert!(!spec(choose(&a).shape).needs.contains(&Know::SteelFrame), "{:?}", choose(&a));
    // and a town that knows nothing at all still gets an answer
    let a = Ask { knows: Knows::NOTHING, ..ask(2000, 8, Lot::Block, Use::Dwellings, 0.5) };
    let _ = choose(&a);
}

#[test]
fn what_the_content_has_is_asked_of_the_town() {
    // the steel frame is in the content: a town knows it when it knows the method, whatever the year
    let k = Knows::of(1950, |m| m != "steel_frame");
    assert!(!k.has(Know::SteelFrame) && k.has(Know::Lift) && k.has(Know::Electricity));
    let k = Knows::of(1890, |m| m == "steel_frame");
    assert!(k.has(Know::SteelFrame) && !k.has(Know::MotorCar) && !k.has(Know::Electricity));
    // and without the content, the years stand in
    assert!(!Knows::by_year(1904).has(Know::Concrete) && Knows::by_year(1905).has(Know::Concrete));
}

#[test]
fn every_form_draws_at_every_level() {
    let pal = Palette::standard();
    for s in FORMS {
        let mut last = 0;
        for detail in Detail::ALL {
            for storeys in [s.storeys.0, s.storeys.1] {
                let f = Form { detail, ..Form::new(s.shape, storeys, (s.years.0 + s.years.1) / 2, Seed::of(3)) };
                let mut m = Model::default();
                form(&mut m, &Site::level(Vec3::ZERO, 0.0), &f, &pal);
                assert!(m.is_sound() && !m.is_empty(), "{} at {:?}", s.name, detail);
                // the same each time
                let mut again = Model::default();
                form(&mut again, &Site::level(Vec3::ZERO, 0.0), &f, &pal);
                assert_eq!(m.solid.pos, again.solid.pos, "{}", s.name);
                if storeys == s.storeys.0 {
                    assert!(m.solid.triangles() >= last, "{} has less to it at {:?}", s.name, detail);
                    last = m.solid.triangles();
                }
            }
        }
        // taller is taller
        let top = |n: u8| {
            let mut m = Model::default();
            form(&mut m, &Site::level(Vec3::ZERO, 0.0), &Form::new(s.shape, n, s.years.1, Seed::of(3)), &pal);
            m.solid.bounds().unwrap().1.y
        };
        if s.storeys.1 > s.storeys.0 + 1 {
            assert!(top(s.storeys.1) > top(s.storeys.0), "{}", s.name);
        }
    }
}
