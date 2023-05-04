// deserted

/*
/// moved into tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteJudgeConfig {
    pub oj: RemoteOJ,
    pub pid: ProblemID,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteOJ {}
*/

/*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub case: ExampleCase,
    pub explanation: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExampleCase {
    Static(StaticExample),
    Dynamic(DynamicExample),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticExample {
    #[serde(rename = "Example Input")]
    pub input: String,
    #[serde(rename = "Example Output")]
    pub output: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicExample {
    #[serde(rename = "Input Data")]
    pub input_path: String,
}
*/
