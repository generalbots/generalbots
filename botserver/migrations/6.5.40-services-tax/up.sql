-- Per-service tax override (issue #722). When 0 the composite rates from
-- `billing_tax_rates` apply; when > 0 it is the effective total tax
-- percentage, split proportionally across IRPJ/CSLL/PIS-COFINS/ISS.

ALTER TABLE services ADD COLUMN IF NOT EXISTS tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0;
