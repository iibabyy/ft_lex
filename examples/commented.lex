%{
/* Définitions de code C incluses au début du fichier lex.yy.c, en dehors de toute fonction */
#include <stdio.h>
#include <string.h>

/* Définition d'une variable globale pour illustrer son utilisation dans les actions */
int mot_compte = 0;
%}

/* Définitions de chaînes de substitution Lex */
CHIFFRE [1-9]
LETTRE [a-zA-Z]
ALPHANUMERIQUE {LETTRE}|{CHIFFRE}
ESPACE [ \t\n]

/* Définition des états de départ */
%s COMMENTAIRE LIGNE_A LIGNE_B LIGNE_C
%x EXCLUSIF

/* Définition des tailles de tables internes (à titre d'exemple) */
%p 3000
%n 600
%a 2500
%e 1200
%k 1100
%o 3500

/* Déclaration du type de yytext (l'un ou l'autre) */
%array
/* %pointer */

%%

    /* Code C inséré au début de la fonction yylex() */
    int ligne_debut_a = 0;
    int ligne_debut_b = 0;
    int ligne_debut_c = 0;

^{LETTRE} {
    /* Action pour le début d'une ligne */
    if (yytext == 'a') {
        BEGIN LIGNE_A;
        ligne_debut_a = 1;
    } else if (yytext == 'b') {
        BEGIN LIGNE_B;
        ligne_debut_b = 1;
    } else if (yytext == 'c') {
        BEGIN LIGNE_C;
        ligne_debut_c = 1;
    } else {
        BEGIN 0; /* Retour à l'état initial */
    }
    ECHO; /* Copier le caractère de début de ligne */
}

<LIGNE_A>magie {
    printf("premier");
}
<LIGNE_B>magie {
    printf("deuxième");
}
<LIGNE_C>magie {
    printf("troisième");
}
<LIGNE_A,LIGNE_B,LIGNE_C>\n {
    BEGIN 0;
    ECHO;
    ligne_debut_a = ligne_debut_b = ligne_debut_c = 0;
}

<EXCLUSIF>fin_exclusif {
    printf("Fin de la section exclusive.\n");
    BEGIN 0;
}
<EXCLUSIF>. {
    /* Consomme tout caractère en état exclusif sans action par défaut (pas de ECHO) */
    printf("[EXCLUSIF: %s]", yytext);
}

debut_exclusif {
    printf("Début de la section exclusive.\n");
    BEGIN EXCLUSIF;
}

"/*" {
    /* Début d'un commentaire multi-ligne */
    BEGIN COMMENTAIRE;
}

<COMMENTAIRE>"*/" {
    /* Fin d'un commentaire multi-ligne */
    BEGIN 0;
}

<COMMENTAIRE>\n {
    /* Ignorer les nouvelles lignes dans les commentaires */
}

<COMMENTAIRE>. {
    /* Ignorer les caractères dans les commentaires */
}

"//".*$ {
    /* Commentaire sur une seule ligne (ignoré) */
}

{ALPHANUMERIQUE}* {
    printf("IDENTIFIANT: %s\n", yytext);
    mot_compte++;
}

{CHIFFRE}+ {
    /* Un nombre entier */
    printf("ENTIER: %s\n", yytext);
}

{CHIFFRE}+"."{CHIFFRE}* {
    /* Un nombre à virgule flottante */
    printf("FLOTTANT: %s\n", yytext);
}

"mot1"|"mot2" {
    /* Plusieurs mots-clés avec la même action */
    printf("MOT-CLÉ: %s\n", yytext);
}

"spécial" {
    /* Action complexe sur plusieurs lignes */
    printf("Mot spécial détecté: %s\n", yytext);
    if (strlen(yytext) > 5) {
        printf("  (long)\n");
    } else {
        printf("  (court)\n");
    }
}

"ou bien" |
"autre choix" {
    printf("Alternative détectée.\n");
}

"plus"'+' {
    printf("Opérateur plus: %s\n", yytext);
}

"moins"- {
    printf("Opérateur moins: %s\n", yytext);
}

"tab"\t {
    printf("Mot suivi d'une tabulation.\n");
}

"fin de ligne" $ {
    printf("Mot à la fin d'une ligne.\n");
}

"^début de ligne" {
    printf("Mot au début d'une ligne.\n");
}

"context"/suivi {
    printf("Mot 'context' suivi de 'suivi'. (Seul 'context' est retourné: %s)\n", yytext);
}

[a-z]+/[1-9]+ {
    printf("Mot suivi d'un nombre (REJET possible).\n");
    /* Exemple d'utilisation de REJECT pour trouver d'autres correspondances */
    if (strcmp(yytext, "test") == 0) {
        printf("  REJET appliqué.\n");
        REJECT;
    }
}

[a-z]+ {
    /* Règle de secours pour les mots (si REJECT n'est pas appelé) */
    printf("AUTRE MOT: %s\n", yytext);
}

{ESPACE}+ {
    /* Ignorer les espaces blancs (action vide) */
}

. {
    /* Règle pour tout autre caractère (affichage et potentielle erreur) */
    printf("CARACTÈRE NON RECONNU: %s\n", yytext);
}

%%
/* Section des sous-routines utilisateur */

int yywrap() {
    /* Fonction appelée à la fin du fichier d'entrée */
    printf("\nAnalyse terminée.\n");
    printf("Nombre total de mots (identifiants) trouvés: %d\n", mot_compte);
    return 1; /* Indique qu'il n'y a plus d'entrée */
}

int main(int argc, char *argv[]) {
    /* Fonction principale */
    printf("Début de l'analyse lexicale...\n");
    if (argc > 1) {
        yyin = fopen(argv[1], "r");
        if (!yyin) {
            perror(argv[1]);
            return 1;
        }
    } else {
        yyin = stdin;
    }
    yylex(); /* Lancer l'analyse lexicale */
    if (yyin != stdin) {
        fclose(yyin);
    }
    return 0;
}