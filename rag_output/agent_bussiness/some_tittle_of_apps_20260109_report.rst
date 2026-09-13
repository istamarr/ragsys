============================================================
RAG Pipeline Analysis Report
============================================================

Project Information
--------------------

:Project Name: some_tittle_of_apps
:Project Path: show flow bussines, flow chart, diagram, describe and analytical, code for it maybe need to fix
:Instructions: structure flow process of  

RAG Response
------------

::

   Maaf, saya tidak bisa menjawab pertanyaan tersebut karena tidak memiliki konteks yang jelas tentang apa yang ingin Anda ulangi. Jika Anda dapat memberikan lebih banyak informasi atau konteks tentang "structure flow process" yang Anda inginkan, saya akan berusaha membantu Anda dengan lebih baik.

RAG Pipeline Flow Chart
-----------------------

::

   
   ┌─────────────────────────────────────────────────────────────┐
   │                    run() in base_chain.rs                   │
   └────────────────────────┬────────────────────────────────────┘
                            │
                            ▼
   ┌─────────────────────────────────────────────────────────────┐
   │              rag_pipeline() in rag_pipeline.rs              │
   │                         │                                   │
   │                         ▼                                   │
   │              build_response_model_rag()                     │
   └────────────────────────┬────────────────────────────────────┘
                            │
             ┌──────────────┴──────────────┐
             ▼                             │
   ┌─────────────────────┐                 │
   │ try_burn_lm_asist() │                 │
   │  (port 9393)        │                 │
   │  /health → /ask     │                 │
   │  [X] FAILED          │                 │
   └─────────┬───────────┘                 │
             │                             │
        ┌────┴────┐                        │
        │ Success?│                        │
        └────┬────┘                        │
             │ No ───────────────────────► ┌┴─────────────────────┐
             │                             │ try_ollama_fallback()│
             ▼                             │ ollamar/tamar:1b     │
       ┌───────────┐                       │ /api/chat            │
       │  Response │◄──────────────────────│ [OK] SUCCESS          │
       └───────────┘                       └──────────────────────┘

Project Analysis Flow Chart
---------------------------

::

   
   ┌─────────────────────────────────────────────────────────────────┐
   │                some_tittle_of_apps Project Flow                 │
   └─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
                     ┌─────────────┐
                     │  ANALYSIS   │
                     │  COMPLETE   │
                     └─────────────┘

