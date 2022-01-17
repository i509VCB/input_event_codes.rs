use std::{collections::BTreeMap, error::Error};

use crate::parse::{Define, Expression};

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub alias_name: String,
    pub value: u32,
    pub comment: Option<String>,
}

#[derive(Debug)]
pub struct Category {
    pub constants: Vec<Constant>,
}

pub fn create_categories(defines: Vec<Define>) -> Result<BTreeMap<&str, Category>, Box<dyn Error>> {
    let mut categories = BTreeMap::new();

    for define in defines {
        let (category_name, constant_name) =
            define.name.split_once('_').expect("Invalid define name");

        if !categories.contains_key(category_name) {
            assert!(categories
                .insert(category_name, Category { constants: vec![] })
                .is_none());
        }

        let category = categories.get_mut(category_name).unwrap();

        // Ensure the name of the constant is a valid rust identifier:
        let name = if constant_name.chars().next().unwrap().is_numeric() {
            format!("_{}", constant_name)
        } else {
            constant_name.to_owned()
        };

        match define.expression {
            Expression::Constant(value) => {
                category.constants.push(Constant {
                    name,
                    alias_name: define.name.to_owned(),
                    value,
                    comment: define.comment.map(ToOwned::to_owned),
                });
            }

            Expression::Expression { .. } => unreachable!(),
        }
    }

    Ok(categories)
}
