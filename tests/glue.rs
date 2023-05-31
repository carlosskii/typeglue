use typeglue::Glue;


#[derive(Glue)]
struct Person {
    name: String
}

#[test]
fn test_glue_basic() {
    let person = Person::from("John".to_string());
    assert_eq!(person.name, "John");
}

#[test]
fn test_glue_backwards() {
    let name = String::from("John");
    let person: Person = name.into();

    let person_name = String::from(person);
    assert_eq!(person_name, "John");
}

#[derive(Glue)]
struct Xyz {
    x: i32,
    y: i32,
    z: i32
}

#[test]
fn test_glue_multifield() {
    let point = Xyz::from((1, 2, 3));

    assert_eq!(point.x, 1);
    assert_eq!(point.y, 2);
    assert_eq!(point.z, 3);
}

#[derive(Glue)]
struct Log(Vec<String>);

#[test]
fn test_glue_tuple() {
    let log = Log::from(vec!["Hello".to_string(), "World".to_string()]);

    assert_eq!(log.0[0], "Hello");
    assert_eq!(log.0[1], "World");
}

#[derive(Glue)]
struct Ticker(String, i32);

#[test]
fn test_glue_tuple_multifield() {
    let ticker = Ticker::from((
        "AAPL".to_string(),
        0
    ));

    assert_eq!(ticker.0, "AAPL");
    assert_eq!(ticker.1, 0);
}

#[derive(Glue)]
struct List<T>(Vec<T>);

#[test]
fn test_glue_generic() {
    let list = List::from(vec![1, 2, 3]);

    assert_eq!(list.0[0], 1);
    assert_eq!(list.0[1], 2);
    assert_eq!(list.0[2], 3);
}

#[derive(Glue)]
enum Number {
    Int(i32),
    Float(f32)
}

#[test]
fn test_glue_enum() {
    let int = Number::from(1);
    let float = Number::from(1.0);

    match int {
        Number::Int(n) => assert_eq!(n, 1),
        _ => unreachable!()
    }

    match float {
        Number::Float(n) => assert_eq!(n, 1.0),
        _ => unreachable!()
    }
}

#[derive(Glue)]
enum Dimensions {
    Vec2(i32, i32),
    Vec3(i32, i32, i32)
}

#[test]
fn test_glue_enum_multifield() {
    let vec2 = Dimensions::from((1, 2));
    let vec3 = Dimensions::from((1, 2, 3));

    match vec2 {
        Dimensions::Vec2(x, y) => {
            assert_eq!(x, 1);
            assert_eq!(y, 2);
        },
        _ => unreachable!()
    }

    match vec3 {
        Dimensions::Vec3(x, y, z) => {
            assert_eq!(x, 1);
            assert_eq!(y, 2);
            assert_eq!(z, 3);
        },
        _ => unreachable!()
    }
}