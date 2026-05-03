#[derive(Clone, Default)]
pub struct FormState {
    pub full_name: String,
    pub project_name: String,
    pub contact_info: String,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
    ReceiveProjectName(FormState),
    ReceiveContactInfo(FormState),
    ReceiveQuestion(FormState),
}
