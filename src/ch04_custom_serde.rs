use serde::{Serialize, Serializer};
use serde::ser::{SerializeStruct, SerializeTupleStruct, SerializeStructVariant, SerializeTupleVariant};

// impl Serialize for i32 {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_i32(*self)
//     }
// }

// impl<T> Serialize for Vec<T>
// where
//     T: Serialize,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut seq = serializer.serialize_seq(Some(self.len()))?;
//         for e in self {
//             seq.serialize_element(e)?;
//         }
//         seq.end()
//     }
// }

// 普通 结构体. 步骤如下:
//   1. serialize_struct
//   2. serialize_field
//   3. end
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Color", 3)?;
        state.serialize_field("r", &self.r)?;
        state.serialize_field("g", &self.g)?;
        state.serialize_field("b", &self.b)?;
        state.end()
    }
}

// 元组 结构体. 步骤如下:
//   1. serialize_tuple_struct
//   2. serialize_field
//   3. end
struct Point2D(f64, f64);

impl Serialize for Point2D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_tuple_struct("Point2D", 2)?;
        state.serialize_field(&self.0)?;
        state.serialize_field(&self.1)?;
        state.end()
    }
}

// newtype struct. 使用 serialize_newtype_struct.
struct Inches(u64);

impl Serialize for Inches {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("Inches", &self.0)
    }
}


// unit struct. 使用 serialize_unit_struct.
struct Instance;


impl Serialize for Instance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit_struct("Instance")
    }
}

#[allow(unused, dead_code)]
enum E {
    // 如下三步:
    //   1. serialize_struct_variant
    //   2. serialize_field
    //   3. end
    Color { r: u8, g: u8, b: u8 },

    // 如下三步:
    //   1. serialize_tuple_variant
    //   2. serialize_field
    //   3. end
    Point2D(f64, f64),

    // 使用 serialize_newtype_variant.
    Inches(u64),

    // 使用 serialize_unit_variant.
    Instance,
}

impl Serialize for E {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            E::Color {r,g,b} => {
                let mut state = serializer.serialize_struct_variant("E", 0, "Color", 3)?;
                state.serialize_field("r", r)?;
                state.serialize_field("g", g)?;
                state.serialize_field("b", b)?;
                state.end()
            }
            E::Point2D(x, y) => {
                let mut state = serializer.serialize_tuple_variant("E", 1, "Point2D",  2)?;
                state.serialize_field(x)?;
                state.serialize_field(y)?;
                state.end()
            }
            E::Inches(i) => serializer.serialize_newtype_variant("E", 2, "Inches", i),
            E::Instance => serializer.serialize_unit_variant("E", 3, "Instance"),
        }
    }
}

#[derive(Serialize)]
struct Efficient<'a> {
    #[serde(with = "serde_bytes")]
    bytes: &'a [u8],
    byte_buf: Vec<u8>,
}

struct Efficient2<'a> {
    bytes: &'a [u8],
    byte_buf: Vec<u8>,
}

impl <'a> Serialize for Efficient2<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
            let mut state = serializer.serialize_struct("Efficient2", 2)?;
        state.serialize_field("bytes", {
            // 对 bytes 添加了一层包装，用来代理调用 `serde_bytes::serialize` 方法
            // 这样就能 使用高效的 字节数组序列化
            struct SerializeWith<'__a, 'a: '__a> {
                values: (&'__a &'a [u8],),
                phantom: serde::export::PhantomData<Efficient2<'a>>,
            }
            impl<'__a, 'a: '__a> serde::Serialize for SerializeWith<'__a, 'a> {
                fn serialize<__S>(
                    &self,
                    __s: __S,
                ) -> serde::export::Result<__S::Ok, __S::Error>
                where
                    __S: serde::Serializer,
                {
                    serde_bytes::serialize(self.values.0, __s)
                }
            }
            &SerializeWith {
                values: (&self.bytes,),
                phantom: serde::export::PhantomData::<Efficient2<'a>>,
            }
        })?;
        state.serialize_field("byte_buf", &self.byte_buf)?;
        state.end()
    }
}

use std::fmt;
use std::marker::PhantomData;
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};

struct MyMap<K, V>(PhantomData<K>, PhantomData<V>);


impl<K, V> MyMap<K, V> {
    fn with_capacity(c: usize) -> Self {
        println!("build MyMap size = {}", c);
        MyMap(PhantomData, PhantomData)
    }

    fn insert(&mut self, _: K, _: V) {
        println!("call MyMap insert")
    }
}

// Visitor 决定不同类型的数据的如何处理，其方法包含默认为报错，调用哪个方法取决于
// Deserializer 的实现和序列化数据流解析
// 针对 Map 类型的 Visitor 需要提供 <K, V> 的泛型，因此需要使用 Deserializer
// PhantomData 用于在零尺寸类型中使用泛型
struct MyMapVisitor<K, V> {
    marker: PhantomData<fn() -> MyMap<K, V>>
}

impl<K, V> MyMapVisitor<K, V> {
    fn new() -> Self {
        MyMapVisitor {
            marker: PhantomData
        }
    }
}

impl<'de, K, V> Visitor<'de> for MyMapVisitor<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    // 表示 这个 Visitor 的产出类型
    type Value = MyMap<K, V>;

    // 格式化一条消息，说明该访客希望接收哪些数据。 
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    // 将 Map 数据 MyMap
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = MyMap::with_capacity(access.size_hint().unwrap_or(0));

        // 遍历数据
        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

// 表示 MyMap 类型可以被反序列化
impl<'de, K, V> Deserialize<'de> for MyMap<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 实例化访问者，并要求反序列化器将其驱动到输入数据上，从而生成MyMap实例。
        deserializer.deserialize_map(MyMapVisitor::new())
    }
}

#[derive(Debug)]
struct Duration {
    secs: u64,
    nanos: u32,
}

impl Duration {
    fn new(secs: u64, nanos: u32) -> Duration {
        Duration {
            secs,
            nanos
        }
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field { Secs, Nanos };

        // 这部分也可以如下写法生成，用于解析结构体的Key
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "secs" => Ok(Field::Secs),
                            "nanos" => Ok(Field::Nanos),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DurationVisitor;

        impl<'de> Visitor<'de> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Duration, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let secs = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let nanos = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Duration::new(secs, nanos))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Duration, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut secs = None;
                let mut nanos = None;
                // next_key 方法将调用 Field 类型的 deserialize 方法
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Secs => {
                            if secs.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            secs = Some(map.next_value()?);
                        }
                        Field::Nanos => {
                            if nanos.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            nanos = Some(map.next_value()?);
                        }
                    }
                }
                let secs = secs.ok_or_else(|| de::Error::missing_field("secs"))?;
                let nanos = nanos.ok_or_else(|| de::Error::missing_field("nanos"))?;
                Ok(Duration::new(secs, nanos))
            }
        }

        // 调用 deserialize_struct 传递 Visitor
        const FIELDS: &'static [&'static str] = &["secs", "nanos"];
        deserializer.deserialize_struct("Duration", FIELDS, DurationVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ch04() {
        // 序列化
        println!("i32 = {}", serde_json::to_string(&1).unwrap());
        println!("Color = {}", serde_json::to_string(&Color{r:1, g:2, b:3}).unwrap());
        println!("Point2D = {}", serde_json::to_string(&Point2D(1.0, 2.0)).unwrap());
        println!("Inches = {}", serde_json::to_string(&Inches(1)).unwrap());
        println!("Instance = {}", serde_json::to_string(&Instance).unwrap());
        println!("E::Color = {}", serde_json::to_string(&E::Color{r:1, g:2, b:3}).unwrap());
        println!("E::Point2D = {}", serde_json::to_string(&E::Point2D(1.0, 2.0)).unwrap());
        println!("E::Inches = {}", serde_json::to_string(&E::Inches(1)).unwrap());
        println!("E::Instance = {}", serde_json::to_string(&E::Instance).unwrap());

        // 反序列化
        let json1 = r#"
        {
            "k1": "v1",
            "k2": "v2"
        }
        "#;
        let _r1: MyMap<String, String> = serde_json::from_str(json1).unwrap();

        let json2 = r#"
        {
            "secs": 1,
            "nanos": 2
        }
        "#;
        let r2: Duration = serde_json::from_str(json2).unwrap();
        println!("Duration = {:?}", r2);
    }
}
