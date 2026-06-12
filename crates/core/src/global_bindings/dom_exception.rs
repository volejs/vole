use rquickjs::Ctx;

pub fn init(ctx: &Ctx) -> Result<(), rquickjs::Error> {
    // Phase 0: DOMException 简化为 Error 子类构造函数
    let ctor: rquickjs::Value = ctx.eval(
        r#"
        function DOMException(message, name) {
            const err = new Error(message !== undefined ? message : "");
            err.name = name !== undefined ? name : "Error";
            const codeMap = {
                "IndexSizeError": 1,
                "HierarchyRequestError": 3,
                "WrongDocumentError": 4,
                "InvalidCharacterError": 5,
                "NoModificationAllowedError": 7,
                "NotFoundError": 8,
                "NotSupportedError": 9,
                "InUseAttributeError": 10,
                "InvalidStateError": 11,
                "SyntaxError": 12,
                "InvalidModificationError": 13,
                "NamespaceError": 14,
                "InvalidAccessError": 15,
                "TypeMismatchError": 17,
                "SecurityError": 18,
                "NetworkError": 19,
                "AbortError": 20,
                "URLMismatchError": 21,
                "QuotaExceededError": 22,
                "TimeoutError": 23,
                "InvalidNodeTypeError": 24,
                "DataCloneError": 25
            };
            err.code = codeMap[err.name] || 0;
            return err;
        }
        DOMException
        "#,
    )?;
    ctx.globals().set("DOMException", ctor)?;
    Ok(())
}
