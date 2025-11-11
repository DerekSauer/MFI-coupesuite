SELECT
    TRIM(UPPER(TO_ASCII(part.prt_no, 'LATIN1'))) AS no_piece,
    TRIM(UPPER(TO_ASCII(part.prt_desc1, 'LATIN1'))) AS desc_piece,
    part.prt_stt_ax1::REAL AS longueur,
    part.prt_stt_ax2::REAL AS largeur,
    part.prt_stt_ax5::REAL AS epaisseur,
    TRIM(part.prt_usr_string1) AS edge_haut,
    TRIM(part.prt_usr_string2) AS edge_droite,
    TRIM(part.prt_usr_string3) AS edge_bas,
    TRIM(part.prt_usr_string4) AS edge_gauche

FROM
    part

WHERE
    -- REMOVE DASHES IN THE INPUT AND TEST TERMS FOR COMPATIBILITY
    -- WITH BARCODE SCANNERS IN THE SHOP
    TRIM(REPLACE(part.prt_no, '-', '')) = TRIM(REPLACE($1, '-', ''))
