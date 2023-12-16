use rkyv::{with::Boxed, Archive, Deserialize, Serialize, rancor::Failure};

// This is the version used by the older client, which can read newer versions
// from senders.
#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
struct ExampleV1 {
    a: i32,
    b: u32,
}

// This is the version used by the newer client, which can send newer versions
// to receivers.
#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
struct ExampleV2 {
    a: i32,
    b: i32,
    c: String,
}

// This wrapper type serializes the contained value out-of-line so that newer
// versions can be viewed as the older version.
//
// In a complete message format, sending a version number along with the buffer
// would allow clients to reject incompatible messages before validating the
// buffer.
#[derive(Archive, Deserialize, Serialize)]
#[repr(transparent)]
#[archive(check_bytes)]
struct Versioned<T>(#[with(Boxed)] pub T);

// This is some code running on the older client. It accepts the older version
// of the struct and prints out the `a` and `b` fields.
fn print_v1(value: &ArchivedExampleV1) {
    println!("v1: a = {}, b = {}", value.a, value.b);
}

// This is some code running on the newer client. It can also print out the `c`
// field for newer versions.
fn print_v2(value: &ArchivedExampleV2) {
    println!("v2: a = {}, b = {}, c = {}", value.a, value.b, value.c);
}

fn main() {
    // These two different versions of the type will be serialized and accessed.
    let v1 = Versioned(ExampleV1 { a: 10, b: 20 });
    let v2 = Versioned(ExampleV2 {
        a: 30,
        b: 50,
        c: "hello world".to_string(),
    });

    // v1 is serialized into v1_bytes
    let v1_bytes =
        rkyv::serialize::<_, 64, Failure>(&v1).expect("failed to serialize v1");
    // v2 is serialized into v2_bytes
    let v2_bytes =
        rkyv::serialize::<_, 64, Failure>(&v2).expect("failed to serialize v2");

    // We can view a v1 as a v1
    let v1_as_v1 =
        rkyv::access::<Versioned<ExampleV1>, Failure>(&v1_bytes).unwrap();
    print_v1(&v1_as_v1.0);

    // We can view a v2 as a v1
    let v2_as_v1 =
        rkyv::access::<Versioned<ExampleV1>, Failure>(&v2_bytes).unwrap();
    print_v1(&v2_as_v1.0);

    // And we can view a v2 as a v2
    let v2_as_v2 =
        rkyv::access::<Versioned<ExampleV2>, Failure>(&v2_bytes).unwrap();
    print_v2(&v2_as_v2.0);

    // But we can't view a v1 as a v2 because v1 is not forward-compatible with v2
    if let Ok(_) = rkyv::access::<Versioned<ExampleV2>, Failure>(&v1_bytes)
    {
        panic!("v1 bytes should not validate as v2");
    } else {
        println!("verified that v1 cannot be viewed as v2");
    }
}
