-- RETURNS A ROW IF THE MODEL NUMBER PASSED IN IS A VALID
-- FURNITURE MODEL. RETURNS NO ROWS IF THE MODEL NUMBER IS INVALID.
SELECT
    TRIM(UPPER(part.prt_no)) AS sku

FROM
    part
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id

WHERE
        TRIM(UPPER(part.prt_no)) = TRIM(UPPER($1))
    AND part_group.pgr_no IN ('765', '860', '890')

LIMIT 1