WITH lamination AS (
    SELECT
        SPLIT_PART(part.prt_no, '-', 1) AS code_matiere,
        MIN((TRIM(part.prt_desc2) != '')::INT
            + (TRIM(part.prt_desc3) != '')::INT) AS faces_laminees

    FROM
        part
        INNER JOIN part_group
                ON part.pgr_id = part_group.pgr_id

    WHERE
        part_group.pgr_no IN ('601', '602', '603', '605', '750', '751')

    GROUP BY
        code_matiere
)

SELECT
    TRIM(UPPER(TO_ASCII(part.prt_no, 'LATIN1'))) AS no_piece,
    TRIM(UPPER(TO_ASCII(part.prt_desc1, 'LATIN1'))) AS desc_piece,
    COALESCE(lamination.faces_laminees) AS face_laminees,
    part.prt_stt_ax1::REAL AS longueur,
    part.prt_stt_ax2::REAL AS largeur,
    part.prt_stt_ax5::REAL AS epaisseur,
    TRIM(part.prt_usr_string1) AS edge_haut,
    TRIM(part.prt_usr_string2) AS edge_droite,
    TRIM(part.prt_usr_string3) AS edge_bas,
    TRIM(part.prt_usr_string4) AS edge_gauche

FROM
    part
    LEFT  JOIN lamination
            ON SPLIT_PART(part.prt_no, '-', 1) = lamination.code_matiere

WHERE
    TRIM(part.prt_no) = $1
