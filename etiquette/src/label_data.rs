/// Data needed to print customer support label.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct LabelData {
    /// SKU.
    pub model_number: String,

    /// The SKU's lot number.
    pub project_number: i32,

    /// SKU's assembly guide #1.
    pub document_1: String,

    /// SKU's assembly guide #2.
    pub document_2: String,

    /// SKU's assembly guide #3.
    pub document_3: String,

    /// Part letter where the customer service label should be affixed.
    pub letter: String,

    /// Number of labels to print.
    pub print_quantity: i32,
}
