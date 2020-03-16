use serde::{Serialize, Deserialize, Serializer};

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct Axis {
    x: i32
}

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[allow(unused, dead_code)]

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Running(i32),
    Stop,
    #[serde(skip)]
    Ready,
    Block,
    #[serde(serialize_with="ser_waiting")]
    Waiting {
        a: i32,
        b: bool,
    },
    // #[serde(other)]
    Unknown,
}

#[allow(unused, dead_code)]
fn ser_waiting<S>(a: &i32, b: &bool, serializer: S) -> Result<S::Ok, S::Error> 
where S: Serializer 
{
    serializer.serialize_str("Waiting")
}

#[derive(Serialize, Deserialize, Debug)]
struct Process {
    state: State
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ch01() {
        let point = Point { x: 1, y: 2 };

        // Rust 类型转换为 JSON 字符串
        let serialized = serde_json::to_string(&point).unwrap();

        // 打印 序列化的字符串 = {"x":1,"y":2}
        println!("point serialized = {}", serialized);
        println!("axis serialized = {}", serde_json::to_string(&Axis{x: 1}).unwrap());
        println!("state.stop serialized = {}", serde_json::to_string(&State::Stop).unwrap());
        println!("state.running serialized = {}", serde_json::to_string(&State::Running(1)).unwrap());
        // println!("state.Ready serialized = {}", serde_json::to_string(&Process { state: State::Ready}).unwrap());
        println!("state.Waiting serialized = {}", serde_json::to_string(&State::Waiting{a:1, b: false}).unwrap());

        // Convert the JSON string back to a Point.
        // 根据 JSON 字符串反序列化 为 Rust 类型
        let deserialized: Point = serde_json::from_str(&serialized).unwrap();

        // 打印 反序列化的字符串 = Point { x: 1, y: 2 }
        println!("deserialized = {:?}", deserialized);
    }
}
