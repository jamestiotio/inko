pub const DEFAULT_GLOBALS: [(&'static str, &'static str); 10] =
    [
        ("core::string::String", "String"),
        ("core::array::Array", "Array"),
        ("core::integer::Integer", "Integer"),
        ("core::float::Float", "Float"),
        ("core::object::Object", "Object"),
        ("core::class::Class", "Class"),
        ("core::trait::Trait", "Trait"),
        ("core::nil::NilSingleton", "nil"),
        ("core::boolean::TrueSingleton", "true"),
        ("core::boolean::FalseSingleton", "false"),
    ];
