//! Query builder utilities for applying filters and sorts
//!
//! This module provides utilities for building database queries with filters and sorting.
//! The actual implementation should be done in the repository layer using Sea-ORM.

use super::query_params::{FilterOperator, FilterParams, SortDirection, SortParams};

/// Helper struct to represent a filter condition for repository layer
#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

impl From<&super::query_params::FilterCondition> for FilterCondition {
    fn from(cond: &super::query_params::FilterCondition) -> Self {
        Self {
            field: cond.field.clone(),
            operator: cond.operator.clone(),
            value: cond.value.clone(),
        }
    }
}

/// Helper struct to represent a sort specification for repository layer
#[derive(Debug, Clone)]
pub struct SortSpec {
    pub field: String,
    pub direction: SortDirection,
}

impl From<&super::query_params::SortSpec> for SortSpec {
    fn from(spec: &super::query_params::SortSpec) -> Self {
        Self {
            field: spec.field.clone(),
            direction: spec.direction.clone(),
        }
    }
}

/// Helper to convert filter params to a list of conditions
pub fn filter_conditions(filter: &FilterParams) -> Vec<FilterCondition> {
    filter
        .conditions
        .iter()
        .map(FilterCondition::from)
        .collect()
}

/// Helper to convert sort params to a list of sort specs
pub fn sort_specs(sort: &SortParams) -> Vec<SortSpec> {
    sort.sorts.iter().map(SortSpec::from).collect()
}
