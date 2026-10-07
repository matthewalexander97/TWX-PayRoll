-- Informational only: preserve exact entered decimals without inventing a financial scale.
ALTER TABLE loads ADD COLUMN mileage TEXT CHECK (
 mileage IS NULL OR (
 length(mileage) BETWEEN 1 AND 128
 AND mileage NOT GLOB '*[^0-9.]*'
 AND substr(mileage,1,1) BETWEEN '0' AND '9'
 AND substr(mileage,-1,1) BETWEEN '0' AND '9'
 AND length(mileage)-length(replace(mileage,'.','')) <= 1
 ));
