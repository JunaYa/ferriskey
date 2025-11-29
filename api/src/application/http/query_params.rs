use std::collections::HashMap;
use std::str::FromStr;

/// Filter operator for query parameters
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterOperator {
    Eq,    // equals (default)
    Ne,    // not equals
    Gt,    // greater than
    Gte,   // greater than or equal
    Lt,    // less than
    Lte,   // less than or equal
    In,    // in list (comma-separated)
    Like,  // like (case-sensitive)
    Ilike, // ilike (case-insensitive, PostgreSQL)
}

impl FromStr for FilterOperator {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "eq" => Ok(FilterOperator::Eq),
            "ne" => Ok(FilterOperator::Ne),
            "gt" => Ok(FilterOperator::Gt),
            "gte" => Ok(FilterOperator::Gte),
            "lt" => Ok(FilterOperator::Lt),
            "lte" => Ok(FilterOperator::Lte),
            "in" => Ok(FilterOperator::In),
            "like" => Ok(FilterOperator::Like),
            "ilike" => Ok(FilterOperator::Ilike),
            _ => Err(()),
        }
    }
}

/// Filter condition for a single field
#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

/// Parsed filter parameters
#[derive(Debug, Clone, Default)]
pub struct FilterParams {
    pub conditions: Vec<FilterCondition>,
}

impl FilterParams {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}

/// Sort direction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Sort specification for a single field
#[derive(Debug, Clone)]
pub struct SortSpec {
    pub field: String,
    pub direction: SortDirection,
}

/// Parsed sort parameters
#[derive(Debug, Clone, Default)]
pub struct SortParams {
    pub sorts: Vec<SortSpec>,
}

impl SortParams {
    pub fn new() -> Self {
        Self { sorts: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.sorts.is_empty()
    }

    /// Parse sort string like "field1,-field2,field3"
    pub fn from_string(s: &str) -> Self {
        let mut sorts = Vec::new();
        for part in s.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some(stripped) = part.strip_prefix('-') {
                sorts.push(SortSpec {
                    field: stripped.to_string(),
                    direction: SortDirection::Desc,
                });
            } else {
                sorts.push(SortSpec {
                    field: part.to_string(),
                    direction: SortDirection::Asc,
                });
            }
        }
        Self { sorts }
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Default)]
pub struct PaginationParams {
    pub offset: i64,
    pub limit: i64,
}

impl PaginationParams {
    pub fn new(offset: Option<i64>, limit: Option<i64>) -> Self {
        Self {
            offset: offset.unwrap_or(0).max(0),
            limit: limit.unwrap_or(20).clamp(1, 100), // Default 20, max 100
        }
    }
}

/// Combined query parameters (filter, sort, pagination)
#[derive(Debug, Clone, Default)]
pub struct QueryParams {
    pub filter: FilterParams,
    pub sort: SortParams,
    pub pagination: PaginationParams,
}

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse from query string map
    /// Handles formats like:
    /// - filter[field]=value (defaults to eq)
    /// - filter[field][operator]=value
    /// - sort=field or sort=-field
    /// - offset=0, limit=20
    pub fn from_query_map(query_map: &HashMap<String, String>) -> Self {
        let mut filter = FilterParams::new();
        let mut sort = SortParams::new();
        let mut offset: Option<i64> = None;
        let mut limit: Option<i64> = None;

        for (key, value) in query_map {
            // Parse filter parameters
            if let Some(filter_key) = key.strip_prefix("filter[") {
                if let Some(end_bracket) = filter_key.find(']') {
                    let field = filter_key[..end_bracket].to_string();
                    let remaining = &filter_key[end_bracket + 1..];

                    if remaining.is_empty() {
                        // filter[field]=value (default to eq)
                        filter.conditions.push(FilterCondition {
                            field,
                            operator: FilterOperator::Eq,
                            value: value.clone(),
                        });
                    } else if remaining.starts_with('[') && remaining.ends_with(']') {
                        // filter[field][operator]=value
                        let operator_str = &remaining[1..remaining.len() - 1];
                        if let Ok(operator) = operator_str.parse::<FilterOperator>() {
                            filter.conditions.push(FilterCondition {
                                field,
                                operator,
                                value: value.clone(),
                            });
                        }
                    }
                }
            }
            // Parse sort parameter
            else if key == "sort" {
                sort = SortParams::from_string(value);
            }
            // Parse pagination parameters
            else if key == "offset" {
                if let Ok(val) = value.parse::<i64>() {
                    offset = Some(val);
                }
            } else if key == "limit"
                && let Ok(val) = value.parse::<i64>()
            {
                limit = Some(val);
            }
        }

        Self {
            filter,
            sort,
            pagination: PaginationParams::new(offset, limit),
        }
    }
}

/// Helper trait for deserializing query parameters with filter/sort support
pub trait QueryParamsExt {
    fn parse_query_params(&self) -> QueryParams;
}

impl QueryParamsExt for HashMap<String, String> {
    fn parse_query_params(&self) -> QueryParams {
        QueryParams::from_query_map(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_parse_simple() {
        let mut map = HashMap::new();
        map.insert("filter[risk_band]".to_string(), "SAFE".to_string());
        let params = QueryParams::from_query_map(&map);
        assert_eq!(params.filter.conditions.len(), 1);
        assert_eq!(params.filter.conditions[0].field, "risk_band");
        assert_eq!(params.filter.conditions[0].operator, FilterOperator::Eq);
        assert_eq!(params.filter.conditions[0].value, "SAFE");
    }

    #[test]
    fn test_filter_parse_with_operator() {
        let mut map = HashMap::new();
        map.insert("filter[risk_score][gte]".to_string(), "50".to_string());
        let params = QueryParams::from_query_map(&map);
        assert_eq!(params.filter.conditions.len(), 1);
        assert_eq!(params.filter.conditions[0].field, "risk_score");
        assert_eq!(params.filter.conditions[0].operator, FilterOperator::Gte);
        assert_eq!(params.filter.conditions[0].value, "50");
    }

    #[test]
    fn test_sort_parse() {
        let mut map = HashMap::new();
        map.insert("sort".to_string(), "-risk_score,created_at".to_string());
        let params = QueryParams::from_query_map(&map);
        assert_eq!(params.sort.sorts.len(), 2);
        assert_eq!(params.sort.sorts[0].field, "risk_score");
        assert_eq!(params.sort.sorts[0].direction, SortDirection::Desc);
        assert_eq!(params.sort.sorts[1].field, "created_at");
        assert_eq!(params.sort.sorts[1].direction, SortDirection::Asc);
    }

    #[test]
    fn test_pagination_parse() {
        let mut map = HashMap::new();
        map.insert("offset".to_string(), "10".to_string());
        map.insert("limit".to_string(), "50".to_string());
        let params = QueryParams::from_query_map(&map);
        assert_eq!(params.pagination.offset, 10);
        assert_eq!(params.pagination.limit, 50);
    }
}
