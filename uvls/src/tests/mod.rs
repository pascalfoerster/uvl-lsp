/**
 * These tests provide high-level coverage but don't need to provide a channel connection to the LSP
 */

use super::*;

#[allow(deprecated)]
const INIT_PARAMS: InitializeParams = InitializeParams { process_id: None, root_path: None, root_uri: None, initialization_options: None,
    capabilities: ClientCapabilities { workspace: None, text_document: None, window: None, general: None, experimental: None }, 
    trace: None, workspace_folders: None, client_info: None, locale: None
};


#[tokio::test]
async fn test_initialize() {
    let lsp = create_backend().await;
    
    let response = lsp.initialize(INIT_PARAMS).await;
    
    assert!(matches!(response, Ok(_)));
    assert_eq!(response.unwrap().server_info, Some(ServerInfo { name: "uvl lsp".to_string(), version: None }));
    /*assert_eq!(response.unwrap(),
    InitializeResult {
        capabilities: ServerCapabilities { position_encoding: None, text_document_sync: Some(Kind(Incremental)), selection_range_provider: None, hover_provider: None, completion_provider: Some(CompletionOptions { resolve_provider: Some(false),trigger_characters: Some([".".to_string()]), all_commit_characters: None, work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None }, completion_item: None }), signature_help_provider: None, definition_provider: Some(Left(true)),type_definition_provider: None, implementation_provider: None, references_provider: Some(Left(true)), document_highlight_provider: None, document_symbol_provider: None, workspace_symbol_provider: None, code_action_provider: None,code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(true) }), document_formatting_provider: None, document_range_formatting_provider: None, document_on_type_formatting_provider: None, rename_provider: Some(Left(true)),document_link_provider: None, color_provider: None, folding_range_provider: None, declaration_provider: None,execute_command_provider: Some(ExecuteCommandOptions { commands: vec!["uvls/show_config".to_string(), "uvls/hide_config".to_string(), "uvls/open_config".to_string(), "uvls/load_config".to_string(), "uvls/generate_diagram".to_string()],work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None } }), workspace: None, call_hierarchy_provider: None,semantic_tokens_provider: Some(SemanticTokensOptions(SemanticTokensOptions { work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },legend: SemanticTokensLegend { token_types: [SemanticTokenType("keyword"), SemanticTokenType("operator"), SemanticTokenType("namespace"), SemanticTokenType("enumMember"), SemanticTokenType("class"), SemanticTokenType("comment"), SemanticTokenType("enum"), SemanticTokenType("interface"), SemanticTokenType("function"), SemanticTokenType("macro"), SemanticTokenType("parameter"), SemanticTokenType("number"), SemanticTokenType("string")],token_modifiers: [SemanticTokenModifier("deprecated"), SemanticTokenModifier("readonly"), SemanticTokenModifier("modification"), SemanticTokenModifier("async"), SemanticTokenModifier("static"), SemanticTokenModifier("abstract"), SemanticTokenModifier("async")] },range: None, full: Some(Delta { delta: Some(true) }) })), moniker_provider: None, inline_value_provider: None, inlay_hint_provider: Some(Left(true)), linked_editing_range_provider: None, experimental: None },
        server_info: Some(ServerInfo { name: "uvl lsp".to_string(), version: None }) }
    );*/
    let response = lsp.initialized(InitializedParams {}).await;
    assert_eq!(response, ());
}

#[tokio::test]
async fn test_did_open() {
    let lsp = create_init_backend().await;
    let _ = open_uvl_file(&lsp, "test.uvl".to_string()).await;
}

#[tokio::test]
async fn test_graph() {
    let lsp = create_init_backend().await;
    let uri = open_uvl_file(&lsp, "generate_diagram.uvl".to_string()).await;

    let result = lsp.execute_command(
        ExecuteCommandParams {
            command: "uvls/generate_diagram".to_string(),
            arguments: vec![serde_json::Value::from(uri.to_string())],
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None } }
    ).await;
    assert!(matches!(result, Ok(_)));

    assert_eq!(
        std::fs::read_to_string(uri.to_file_path().unwrap().with_file_name("generate_diagram.dot")).unwrap(),
        std::fs::read_to_string(uri.to_file_path().unwrap().with_file_name("generate_diagram_expected.dot")).unwrap(),
    );
}
#[tokio::test]
async fn test_goto_definition() {
    let lsp = create_init_backend().await;
    let uri = open_uvl_file(&lsp, "goto_definition.uvl".to_string()).await;

    fn goto_definition_params(line: u32, character: u32, uri: Url) -> GotoDefinitionParams {
        GotoDefinitionParams { 
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character }
            },
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            partial_result_params: PartialResultParams { partial_result_token: None } 
        }
    }

    // Find Feature1 Definition
    let response = lsp.goto_definition(goto_definition_params(5, 5, uri.clone())).await;
    assert!(matches!(response, Ok(_)));
    assert!(matches!(response.clone().unwrap(), Some(_)));
    assert_eq!(
        response.unwrap().unwrap(),
        GotoDefinitionResponse::Array(
            vec![
                Location::new(uri.clone(), Range { start: Position { line: 1, character: 4 }, end: Position { line: 1, character: 12 } })
            ]
        )
    );
    // Find Feature2 Definition
    let response = lsp.goto_definition(goto_definition_params(6, 5, uri.clone())).await;
    assert!(matches!(response, Ok(_)));
    assert!(matches!(response.clone().unwrap(), Some(_)));
    assert_eq!(
        response.unwrap().unwrap(),
        GotoDefinitionResponse::Array(
            vec![
                Location::new(uri.clone(), Range { start: Position { line: 2, character: 4 }, end: Position { line: 2, character: 12 } })
            ]
        )
    );
    // Find Feature2.attr Definition
    let response = lsp.goto_definition(goto_definition_params(6, 14, uri.clone())).await;
    assert!(matches!(response, Ok(_)));
    assert!(matches!(response.clone().unwrap(), Some(_)));
    assert_eq!(
        response.unwrap().unwrap(),
        GotoDefinitionResponse::Array(
            vec![
                Location::new(uri.clone(), Range { start: Position { line: 2, character: 14 }, end: Position { line: 2, character: 18 } })
            ]
        )
    );
}

#[tokio::test]
async fn test_completion() {
    let lsp = create_init_backend().await;
    let uri = open_uvl_file(&lsp, "completion_1.uvl".to_string()).await;
    
    fn completion_params(line: u32, character: u32, uri: Url) -> CompletionParams {
        CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character} },
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            partial_result_params: PartialResultParams { partial_result_token: None },
            context: Some(CompletionContext { trigger_kind: CompletionTriggerKind::INVOKED, trigger_character: None })
        }
    }

    async fn check_completions(expected: Vec<&str>, lsp: &Backend, uri: &Url, line: u32, character: u32) {
        let response = lsp.completion(completion_params(line, character, uri.clone())).await;
        assert!(matches!(response, Ok(_)));
        assert!(matches!(response.clone().unwrap(), Some(_)));

        if let CompletionResponse::List(completions) = response.clone().unwrap().unwrap() {
            let labels: Vec<&String> = completions.items.iter().map(|c| &c.label ).collect();
            for e in expected {
                assert!(labels.contains(&&e.to_string()));
            }
        } else {
            assert!(false);
        }
    }
    
    // Root completion
    check_completions(
        vec!["namespace", "imports", "include", "features", "constraints"],
        &lsp, &uri, 0, 0
    ).await;

    check_completions(
        vec!["cardinality"],
        &lsp, &uri, 1, 13
    ).await;

    check_completions(
        vec!["mandatory", "optional", "alternative", "or"],
        &lsp, &uri, 4, 8
    ).await;

    check_completions(
        vec!["&", "<", ">", "|", "<=>", "==", "=>"],
        &lsp, &uri, 6, 13
    ).await;

    check_completions(
        vec!["Feature1", "Feature2", "Feature2.attr", "avg", "sum", "len", "ceil", "floor"],
        &lsp, &uri, 9, 4
    ).await;


}
#[tokio::test]
async fn test_references() {
    let lsp = create_init_backend().await;
    let uri = open_uvl_file(&lsp, "references.uvl".to_string()).await;

    fn reference_params(line: u32, character: u32, uri: Url) -> ReferenceParams {
        ReferenceParams { text_document_position:
            TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character} },
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            partial_result_params: PartialResultParams { partial_result_token: None },
            context: ReferenceContext { include_declaration: true }
        }
    }

    // Find Feature1 References
    let response = lsp.references(reference_params(1, 4, uri.clone())).await;
    assert!(matches!(response, Ok(_)));
    assert!(matches!(response.clone().unwrap(), Some(_)));
    assert_eq!(
        response.unwrap().unwrap(),
        vec![
            Location::new(uri.clone(), Range { start: Position { line: 5, character: 4 }, end: Position { line: 5, character: 12 } }),
            Location::new(uri.clone(), Range { start: Position { line: 7, character: 4 }, end: Position { line: 7, character: 12 } }),
        ]
    );

    // Find Feature2 Reference
    let response = lsp.references(reference_params(2, 4, uri.clone())).await;
    assert!(matches!(response, Ok(_)));
    assert!(matches!(response.clone().unwrap(), Some(_)));
    assert_eq!(
        response.unwrap().unwrap(),
        vec![Location::new(uri.clone(), Range { start: Position { line: 6, character: 4 }, end: Position { line: 6, character: 12 } })]
    );

    // Find Feature2.attr Reference 
    let response = lsp.references(reference_params(2, 16, uri.clone())).await;
    assert!(matches!(response, Ok(_)));
    assert!(matches!(response.clone().unwrap(), Some(_)));
    assert_eq!(
        response.unwrap().unwrap(),
        vec![Location::new(uri.clone(), Range { start: Position { line: 6, character: 13 }, end: Position { line: 6, character: 17 } })]
    );

    // Don't find F3 References 
    let response = lsp.references(reference_params(3, 4, uri.clone())).await;
    assert!(matches!(response, Ok(_)));
    assert!(matches!(response.clone().unwrap(), Some(_)));
    assert_eq!(
        response.unwrap().unwrap(),
        vec![]
    );
}

async fn open_uvl_file(lsp: &Backend, file_name: String) -> Url {
    let path = append_to_path(std::env::current_dir().unwrap(), format!("/src/tests/uvl_files/{}", file_name));
    let response = lsp.did_open(
        DidOpenTextDocumentParams { text_document: TextDocumentItem {
            uri: Url::from_file_path(path.clone()).unwrap(), language_id: "uvl".to_string(), version: 1,
            text: std::fs::read_to_string(path.clone()).unwrap()
          } }
    ).await;
    assert_eq!(response, ());
    let _ = lsp.pipeline.sync_root_global().await; // wait for the RootGraph to update

    let uri = Url::from_file_path(path).unwrap();
    lsp.load(uri.clone());

    uri
}

async fn create_init_backend() -> Backend {
    let lsp = create_backend().await;
    
    let response = lsp.initialize(INIT_PARAMS).await;
    assert!(matches!(response, Ok(_)));
    let response = lsp.initialized(InitializedParams {}).await;
    assert_eq!(response, ());
    lsp
}

async fn create_backend() -> Backend {
    let mut backend = None;
    let (_service, _socket) = LspService::new(|client| {
        let pipeline = AsyncPipeline::new(client.clone());
        info!("create service");
        let port = 3000;
        backend = Some(Backend {
            settings: parking_lot::Mutex::new(Settings::default()),
            web_handler_uri: format!("http://localhost:{port}"),
            pipeline: pipeline.clone(),
            coloring: Arc::new(ide::color::State::new()),
            client: client.clone(),
        });
        Backend {
            settings: parking_lot::Mutex::new(Settings::default()),
            web_handler_uri: format!("http://localhost:{port}"),
            pipeline,
            coloring: Arc::new(ide::color::State::new()),
            client,
        }
    });

    assert!(backend.is_some());
    backend.unwrap()
}

fn append_to_path(p: impl Into<std::ffi::OsString>, s: impl AsRef<std::ffi::OsStr>) -> PathBuf {
    let mut p = p.into();
    p.push(s);
    p.into()
}

// Based on https://github.com/ebkalderon/tower-lsp/issues/355
/*
const INITIALIZE: &str = r#"{"jsonrpc":"2.0","method":"initialize", "params": {"capabilities":{}}, "id":1}"#;


fn start_server() -> (tokio::io::DuplexStream, tokio::io::DuplexStream) {
    let (req_client, req_server) = tokio::io::duplex(1024);
    let (resp_server, resp_client) = tokio::io::duplex(1024);
    let (service, socket) = LspService::new(|client| {
        let pipeline = AsyncPipeline::new(client.clone());
        info!("create service");
        let port = 3000;
        Backend {
            settings: parking_lot::Mutex::new(Settings::default()),
            web_handler_uri: format!("http://localhost:{port}"),
            pipeline,
            coloring: Arc::new(ide::color::State::new()),
            client,
    }});
    tokio::spawn(Server::new(req_server, resp_server, socket).serve(service));
    //rt.spawn(Server::new(req_server, resp_server, socket).serve(service));
    (req_client, resp_client)
}

fn req(msg: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg)
}

fn body(src: &[u8]) -> std::result::Result<&str, anyhow::Error>  {
    // parse headers to get headers length
    let mut dst = [httparse::EMPTY_HEADER; 2];

    let (headers_len, _) = match httparse::parse_headers(src, &mut dst)? {
        httparse::Status::Complete(output) => output,
        httparse::Status::Partial => return Err(anyhow::anyhow!("partial headers")),
    };

    // skip headers
    let skipped = &src[headers_len..];

    // return the rest (ie: the body) as &str
    std::str::from_utf8(skipped).map_err(anyhow::Error::from)
}
*/
