WITH documentSelector AS
(
SELECT
    project_material.prj_id,
    ARRAY_AGG(UPPER(TRIM(project_material.prt_no))) AS documentList

FROM
    project_material
    INNER JOIN part
            ON project_material.prt_id = part.prt_id
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id

WHERE
    part_group.pgr_no = '665'

GROUP BY
    project_material.prj_id
)

SELECT
	UPPER(TRIM(REPLACE(project.prt_no, '90-', ''))) AS model_Number,
	project.prj_no::INT                             AS project_Number,
	COALESCE(documentSelector.documentList[1], '')  AS document_1,
	COALESCE(documentSelector.documentList[2], '')  AS document_2,
	COALESCE(documentSelector.documentList[3], '')  AS document_3,
	TRIM(UPPER(part.prt_idx1_2))                    AS letter,
    project.prj_req_qty::INT                        AS print_quantity

FROM
    project
	INNER JOIN part
            ON project.prt_id = part.prt_id
	INNER JOIN documentSelector
            ON project.prj_id = documentSelector.prj_id

WHERE
    project.prj_no = $1