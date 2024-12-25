#[derive(Debug, Default)]
pub struct MetamathData {
    pub constants: Vec<Constant>,
    pub variables: Vec<Variable>,
    pub theorems: Vec<Theorem>,
    pub in_progress_theorems: Vec<InProgressTheorem>,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct Theorem {
    pub name: String,
    pub description: String,
    pub disjoints: Vec<String>,
    pub hypotheses: Vec<Hypothesis>,
    pub assertion: String,
    pub proof: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub label: String,
    pub hypothesis: String,
}

#[derive(Debug, Clone)]
pub struct InProgressTheorem {
    pub name: String,
    pub text: String,
}

impl serde::Serialize for MetamathData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("MetamathData", 4)?;
        state.serialize_field("constants", &self.constants)?;
        state.serialize_field("variables", &self.variables)?;
        state.serialize_field("theorems", &self.theorems)?;
        state.serialize_field("in_progress_theorems", &self.in_progress_theorems)?;
        state.end()
    }
}

impl serde::Serialize for Constant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Constant", 1)?;
        state.serialize_field("symbol", &self.symbol)?;
        state.end()
    }
}

impl serde::Serialize for Variable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Variable", 1)?;
        state.serialize_field("symbol", &self.symbol)?;
        state.end()
    }
}

impl serde::Serialize for Theorem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("InProgressTheorem", 6)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("disjoints", &self.disjoints)?;
        state.serialize_field("hypotheses", &self.hypotheses)?;
        state.serialize_field("assertion", &self.assertion)?;
        state.serialize_field("proof", &self.proof)?;
        state.end()
    }
}

impl serde::Serialize for Hypothesis {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("InProgressTheorem", 2)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("hypothesis", &self.hypothesis)?;
        state.end()
    }
}

impl serde::Serialize for InProgressTheorem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("InProgressTheorem", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("text", &self.text)?;
        state.end()
    }
}
