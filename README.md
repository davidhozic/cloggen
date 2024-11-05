# CLOGGEN
Generator študentskih mnenj (za habilitacijo).

## Namestitev pisave (font)
Generiran dokument uporablja pisavo *Roboto*. V primeru, da pisava na sistemu ni nameščena, se dokument ne bo generiral.
Za namestitev uporabi datoteke v mapi ``data/fonts/Roboto``. Namesti vse datoteke.

## Uporaba

Za generiranje dokumenta uporabi ukaz:

    cloggen create <CSV DATOTEKA STUDIS ANKET> <JSON NABOR ODZIVOV> <TEX DOKUMENT>   

- ``<CSV DATOTEKA STUDIS ANKET>`` predstavlja izvoženo CSV datoteko z ocenami kandidata za posamezno vprašanje STUDIS anket
- ``<TEX DOKUMENT>`` predstavlja, in ``<JSON NABOR ODZIVOV>`` predstavlja JSON datoteko, ki definira odgovore za posamezno mejo ocene v formatu:
    ```json
        {
        "Vprašanje": {
            "Gledano v celoti, je delo izvajalca/ke kakovostno.": {
                "1": ["Odziv 1", "Odziv 2", ...],
                "1.5": ["Odziv 1", "Odziv 2", ...],
                ...
                "4": ["Odziv 1", "Odziv 2", ...],
                "4.5": ["Odziv 1", "Odziv 2", ...],
            }
        }
    }
    ```

    Odzivi so razporejeni po večih številkah. Številke so minimalna meja povprečne ocene pri posameznem vprašanju, ki
    jo mora kandidat imeti, zato da dobi enega izmed pripadajočih odzivov.
    
    Odziv bo izbran iz možnih odzivov, ki pripadajo prvi manjši oceni od povprečne ocene kandidata. Na primer, če ima
    kandidat pri vprašanju *Gledano v celoti, je delo izvajalca/ke kakovostno.* povprečno oceno 4.3, bo ob uporabi
    zgornjega JSON primera odziv izbran iz odzivov, ki pripadajo oceni 4.0 (``"4": ["Odziv 1", "Odziv 2", ...]``)
- ``<TEX DOKUMENT>`` predstavlja glavni LaTeX dokument (datoteko),
    ki bo uporabljen za generacijo izhodnega mnenja v PDF obliki.
    Dokument mora vsebovati ``{AUTO_GEN}`` tekst, ki predstavlja lokacijo
    vstavitve odzivov/odgovorov, generiranih iz zgornje JSON datoteke odzivov.

### Popoln primer zagona
``cloggen create ocena.csv mnenje.json data/mnenje.tex``
