diesel::table! {
    budget.records (id){
        id -> BigInt,
        amount -> Double,
        date -> Date,
        card -> Text,
        description -> Text,
        event_time -> Timestamptz,
    }
}

diesel::table! {
    budget.description_information (description) {
        description -> Text,
        primary_information -> Nullable<Text>,
        secondary_information -> Nullable<Text>,
        tertiary_information -> Nullable<Text>,
        additional_information -> Nullable<Text>,
    }
}
