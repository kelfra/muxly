-- Revert Router configuration migration

DROP TABLE IF EXISTS route_rules;
DROP TABLE IF EXISTS routes;
DROP TABLE IF EXISTS rule_destinations;
DROP TABLE IF EXISTS rule_transformations;
DROP TABLE IF EXISTS routing_rules;
DROP TABLE IF EXISTS destinations; 