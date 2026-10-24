use novasce_casa::nucleo::*;

#[test]
fn contrato_le_o_json_do_indice() {
    let c: Comecar = serde_json::from_str(
        r#"{
        "firstName":"Luan","isNew":true,"returning":false,
        "area":{"name":"Comercial","ownerName":"Eduardo","docUrl":"https://t/x"},
        "firstTask":{"id":"t1","title":"Conhecer o SOPEC","dueAt":null,"url":"https://t/t1"},
        "kit":{"available":false},
        "spaces":[{"slug":"studio","name":"Studio","repo":"Novasce/studio","member":true}],
        "emailSignatureHtml":"<p>Luan</p>"}"#,
    )
    .unwrap();
    assert!(c.is_new);
    assert_eq!(c.spaces[0].slug, "studio");
    let m: Maquina = serde_json::from_str(r#"{"id":"m1","name":"Notebook do Luan","role":"notebook","owner":{"id":"u1","name":"Luan Silva","firstName":"Luan","email":"l@n.com","isNew":false},"patrimonyCode":"NB-014"}"#).unwrap();
    let dono: &DonoMaquina = m.owner.as_ref().unwrap();
    assert_eq!(dono.first_name, "Luan");
    let resposta: RespostaChamado = serde_json::from_str(
        r#"{"id":"r1","type":"SOLUCAO","body":"Tente de novo.","createdAt":"2026-10-23T14:00:00Z"}"#,
    )
    .unwrap();
    assert!(matches!(resposta.tipo, TipoResposta::Solucao));
    let d: DeviceInicio = serde_json::from_str(r#"{"userCode":"ABCD-EFGH","verificationUri":"https://t/os/entrar","deviceCode":"dc","interval":5,"expiresIn":600}"#).unwrap();
    assert_eq!(d.user_code, "ABCD-EFGH");
    let t: DeviceToken = serde_json::from_str(r#"{"userToken":"nvos_u_x","user":{"id":"u","name":"Luan Silva","firstName":"Luan","email":"l@n.com","role":"MEMBER"}}"#).unwrap();
    let u: &UsuarioResumo = &t.user;
    assert_eq!(u.first_name, "Luan");
}

#[test]
fn config_e_token_do_m1() {
    let c = Config::de_texto("tenancy_api = \"https://api.exemplo\"\ntenancy_web = \"https://web.exemplo\"\nsala_url = \"${TENANCY_WEB}/sala\"\n", None).unwrap();
    assert_eq!(c.tenancy_web, "https://web.exemplo");
    let d = tempfile::tempdir().unwrap();
    assert!(ler_maquina(&d.path().join("maquina.json")).is_err());
    let _ = CAMINHO_MAQUINA;
}
