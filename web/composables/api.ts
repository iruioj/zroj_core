// generated by server/src/bin/gen_docs.rs
// DO NOT EDIT.

import { callAPI, fetchAPI, type ExtAsyncData } from "./inner/fetch";

/**
 GFM: alignment of phrasing content.

 Used to align the contents of table cells within a table.
*/
export type AlignKind = ( AlignKindCenter | AlignKindLeft | AlignKindNone | AlignKindRight );
/**
 Center alignment.

 See the `center` value of the `text-align` CSS property.

 ```markdown
   | | aaa |
 > | | :-: |
       ^^^
 ```
*/
export type AlignKindCenter = "center";
/**
 Left alignment.

 See the `left` value of the `text-align` CSS property.

 ```markdown
   | | aaa |
 > | | :-- |
       ^^^
 ```
*/
export type AlignKindLeft = "left";
/**
 No alignment.

 Phrasing content is aligned as defined by the host environment.

 ```markdown
   | | aaa |
 > | | --- |
       ^^^
 ```
*/
export type AlignKindNone = "none";
/**
 Right alignment.

 See the `right` value of the `text-align` CSS property.

 ```markdown
   | | aaa |
 > | | --: |
       ^^^
 ```
*/
export type AlignKindRight = "right";
/**
*/
export type AuthInfoRes = {
    email: string;
    username: Username;
};
/**
 Block quote.

 ```markdown
 > | > a
     ^^^
 ```
*/
export type BlockQuote = {
    children: Node[];
};
/**
 Break.

 ```markdown
 > | a\
      ^
   | b
 ```
*/
export type Break = {
};
/**
 Code (flow).

 ```markdown
 > | ~~~
     ^^^
 > | a
     ^
 > | ~~~
     ^^^
 ```
*/
export type Code = {
    lang: ( undefined | null | string );
    meta: ( undefined | null | string );
    value: string;
};
/**
*/
export type ContestMeta = {
    duration: Elapse;
    end_time: number;
    id: number;
    start_time: number;
    title: string;
};
/**
*/
export type CtstMetasQuery = {
    max_count: number;
    offset: number;
    pattern: ( undefined | null | string );
};
/**
*/
export type CustomTestResult = {
    result: ( undefined | TaskReport | null );
};
/**
 Definition.

 ```markdown
 > | [a]: b
     ^^^^^^
 ```
*/
export type Definition = {
    identifier: string;
    label: ( undefined | null | string );
    title: ( undefined | null | string );
    url: string;
};
/**
 GFM: delete.

 ```markdown
 > | ~~a~~
     ^^^^^
 ```
*/
export type Delete = {
    children: Node[];
};
/**
*/
export type DetailQuery = {
    sid: number;
};
/**
*/
export type DetailReturn = {
    info: SubmInfo;
    judge: ( undefined | null | string[] );
};
/**
 时间表示，数值单位为 ms
*/
export type Elapse = number;
/**
 Emphasis.

 ```markdown
 > | *a*
     ^^^
 ```
*/
export type Emphasis = {
    children: Node[];
};
/**
 描述一个文件
*/
export type FileDescriptor = ( "Stdin" | "Stdout" | { Named: string; } );
/**
 内置的支持的文件类型
*/
export type FileType = (
    FileTypeAssembly
    | FileTypeGnuCpp14O2
    | FileTypeGnuCpp17O2
    | FileTypeGnuCpp20O2
    | FileTypePlain
    | FileTypePython
    | FileTypeRust
);
/**
*/
export type FileTypeAssembly = "gnu_assembly";
/**
*/
export type FileTypeGnuCpp14O2 = "gnu_cpp14_o2";
/**
*/
export type FileTypeGnuCpp17O2 = "gnu_cpp17_o2";
/**
*/
export type FileTypeGnuCpp20O2 = "gnu_cpp20_o2";
/**
*/
export type FileTypePlain = "plain";
/**
*/
export type FileTypePython = "python3";
/**
*/
export type FileTypeRust = "rust";
/**
 GFM: footnote definition.

 ```markdown
 > | [^a]: b
     ^^^^^^^
 ```
*/
export type FootnoteDefinition = {
    children: Node[];
    identifier: string;
    label: ( undefined | null | string );
};
/**
 GFM: footnote reference.

 ```markdown
 > | [^a]
     ^^^^
 ```
*/
export type FootnoteReference = {
    identifier: string;
    label: ( undefined | null | string );
};
/**
*/
export type FullDataMetaQuery = {
    id: number;
};
/**
*/
export type FullJudgeReport = {
    data: ( undefined | JudgeReport | null );
    extra: ( undefined | JudgeReport | null );
    pre: ( undefined | JudgeReport | null );
};
/**
 Gender type

 TODO: 更多的性别
*/
export type Gender = ( GenderFemale | GenderMale | GenderOthers | GenderPrivate );
/**
*/
export type GenderFemale = "Female";
/**
*/
export type GenderMale = "Male";
/**
*/
export type GenderOthers = "Others";
/**
*/
export type GenderPrivate = "Private";
/**
*/
export type GravatarInfo = {
    email: string;
    no_cache: ( undefined | null | boolean );
};
/**
 Heading.

 ```markdown
 > | # a
     ^^^
 ```
*/
export type Heading = {
    children: Node[];
    depth: number;
};
/**
 Html (flow or phrasing).

 ```markdown
 > | <a>
     ^^^
 ```
*/
export type Html = {
    value: string;
};
/**
 for traditional problem
*/
export type IOKind = ( "StdIO" | { FileIO: { input: FileDescriptor; output: FileDescriptor; }; } );
/**
 Image.

 ```markdown
 > | ![a](b)
     ^^^^^^^
 ```
*/
export type Image = {
    alt: string;
    title: ( undefined | null | string );
    url: string;
};
/**
 Image reference.

 ```markdown
 > | ![a]
     ^^^^
 ```
*/
export type ImageReference = {
    alt: string;
    identifier: string;
    label: ( undefined | null | string );
    reference_kind: ReferenceKind;
};
/**
 Code (phrasing).

 ```markdown
 > | `a`
     ^^^
 ```
*/
export type InlineCode = {
    value: string;
};
/**
 Math (phrasing).

 ```markdown
 > | $a$
     ^^^
 ```
*/
export type InlineMath = {
    value: string;
};
/**
*/
export type JudgeDetail = ( JudgeDetailSubtask | JudgeDetailTests );
/**
*/
export type JudgeDetailSubtask = {
    tasks: SubtaskReport[];
    type: "Subtask";
};
/**
*/
export type JudgeDetailTests = {
    tasks: ( undefined | TaskReport | null )[];
    type: "Tests";
};
/**
*/
export type JudgeReport = {
    detail: JudgeDetail;
    meta: TaskMeta;
};
/**
*/
export type JudgeReturn = {
    sid: number;
};
/**
 一个测试点提交的可能的返回状态
*/
export type JudgerStatus = (
    JudgerStatusCompileError
    | JudgerStatusGood
    | JudgerStatusMemoryLimitExceeded
    | JudgerStatusRuntimeError
    | JudgerStatusTimeLimitExceeded
    | JudgerStatusWrongAnswer
);
/**
 编译错误
*/
export type JudgerStatusCompileError = {
    name: "compile_error";
    payload: ( undefined | SandboxStatus | null );
};
/**
 目前没有问题。不等价于通过（得看得分是否等于总分）
*/
export type JudgerStatusGood = {
    name: "good";
    payload: null;
};
/**
 超出内存限制
*/
export type JudgerStatusMemoryLimitExceeded = {
    name: "memory_limit_exceeded";
    payload: null;
};
/**
*/
export type JudgerStatusRuntimeError = {
    name: "runtime_error";
    payload: null;
};
/**
*/
export type JudgerStatusTimeLimitExceeded = {
    name: "time_limit_exceeded";
    payload: null;
};
/**
*/
export type JudgerStatusWrongAnswer = {
    name: "wrong_answer";
    payload: null;
};
/**
 Link.

 ```markdown
 > | [a](b)
     ^^^^^^
 ```
*/
export type Link = {
    children: Node[];
    title: ( undefined | null | string );
    url: string;
};
/**
 Link reference.

 ```markdown
 > | [a]
     ^^^
 ```
*/
export type LinkReference = {
    children: Node[];
    identifier: string;
    label: ( undefined | null | string );
    reference_kind: ReferenceKind;
};
/**
 List.

 ```markdown
 > | * a
     ^^^
 ```
*/
export type List = {
    children: Node[];
    ordered: boolean;
    spread: boolean;
    start: ( undefined | null | number );
};
/**
 List item.

 ```markdown
 > | * a
     ^^^
 ```
*/
export type ListItem = {
    checked: ( undefined | null | boolean );
    children: Node[];
    spread: boolean;
};
/**
 format of login payload
*/
export type LoginPayload = {
    passwordHash: string;
    username: Username;
};
/**
 Math (flow).

 ```markdown
 > | $$
     ^^
 > | a
     ^
 > | $$
     ^^
 ```
*/
export type Math = {
    meta: ( undefined | null | string );
    value: string;
};
/**
 内存空间表示，数值单位为 byte
*/
export type Memory = number;
/**
 Nodes.
*/
export type Node = (
    NodeBlockQuote
    | NodeBreak
    | NodeCode
    | NodeDefinition
    | NodeDelete
    | NodeEmphasis
    | NodeFootnoteDefinition
    | NodeFootnoteReference
    | NodeHeading
    | NodeHtml
    | NodeImage
    | NodeImageReference
    | NodeInlineCode
    | NodeInlineMath
    | NodeLink
    | NodeLinkReference
    | NodeList
    | NodeListItem
    | NodeMath
    | NodeParagraph
    | NodeRoot
    | NodeStrong
    | NodeTable
    | NodeTableCell
    | NodeTableRow
    | NodeText
    | NodeThematicBreak
    | NodeToml
    | NodeTwoColumns
    | NodeYaml
);
/**
 Block quote.
*/
export type NodeBlockQuote = ( BlockQuote & { type: "blockquote"; } );
/**
 Break.
*/
export type NodeBreak = ( Break & { type: "break"; } );
/**
 Code (flow).
*/
export type NodeCode = ( Code & { type: "code"; } );
/**
 Definition.
*/
export type NodeDefinition = ( Definition & { type: "definition"; } );
/**
 Delete.
*/
export type NodeDelete = ( Delete & { type: "delete"; } );
/**
 Emphasis.
*/
export type NodeEmphasis = ( Emphasis & { type: "emphasis"; } );
/**
 Footnote definition.
*/
export type NodeFootnoteDefinition = ( FootnoteDefinition & { type: "footnoteDefinition"; } );
/**
 Footnote reference.
*/
export type NodeFootnoteReference = ( FootnoteReference & { type: "footnoteReference"; } );
/**
 Heading.
*/
export type NodeHeading = ( Heading & { type: "heading"; } );
/**
 Html (phrasing).
*/
export type NodeHtml = ( Html & { type: "html"; } );
/**
 Image.
*/
export type NodeImage = ( Image & { type: "image"; } );
/**
 Image reference.
*/
export type NodeImageReference = ( ImageReference & { type: "imageReference"; } );
/**
 Code (phrasing).
*/
export type NodeInlineCode = ( InlineCode & { type: "inlineCode"; } );
/**
 Math (phrasing).
*/
export type NodeInlineMath = ( InlineMath & { type: "inlineMath"; } );
/**
 Link.
*/
export type NodeLink = ( Link & { type: "link"; } );
/**
 Link reference.
*/
export type NodeLinkReference = ( LinkReference & { type: "linkReference"; } );
/**
 List.
*/
export type NodeList = ( List & { type: "list"; } );
/**
 List item.
*/
export type NodeListItem = ( ListItem & { type: "listItem"; } );
/**
 Math (flow).
*/
export type NodeMath = ( Math & { type: "math"; } );
/**
 Paragraph.
*/
export type NodeParagraph = ( Paragraph & { type: "paragraph"; } );
/**
 Root.
*/
export type NodeRoot = ( Root & { type: "root"; } );
/**
 Strong
*/
export type NodeStrong = ( Strong & { type: "strong"; } );
/**
 Html (flow).
 Table.
*/
export type NodeTable = ( Table & { type: "table"; } );
/**
 Table cell.
*/
export type NodeTableCell = ( TableCell & { type: "tableCell"; } );
/**
 Table row.
*/
export type NodeTableRow = ( TableRow & { type: "tableRow"; } );
/**
 Text.
*/
export type NodeText = ( Text & { type: "text"; } );
/**
 Thematic break.
*/
export type NodeThematicBreak = ( ThematicBreak & { type: "thematicBreak"; } );
/**
 Toml.
*/
export type NodeToml = ( Toml & { type: "toml"; } );
/**
 Two columns.
*/
export type NodeTwoColumns = ( TwoColumns & { type: "twoColumns"; } );
/**
 Yaml.
*/
export type NodeYaml = ( Yaml & { type: "yaml"; } );
/**
 Paragraph.

 ```markdown
 > | a
     ^
 ```
*/
export type Paragraph = {
    children: Node[];
};
/**
*/
export type PostDataReturn = {
    id: number;
};
/**
*/
export type ProbMetasQuery = {
    max_count: number;
    offset: number;
    pattern: ( undefined | null | string );
};
/**
*/
export type ProblemKind = ( "Interactive" | "SubmitAnswer" | { Traditional: IOKind; } );
/**
*/
export type ProblemMeta = {
    id: number;
    tags: string;
    title: string;
};
/**
*/
export type ProfileQuery = {
    username: Username;
};
/**
 Explicitness of a reference.
*/
export type ReferenceKind = ( ReferenceKindCollapsed | ReferenceKindFull | ReferenceKindShortcut );
/**
 The reference is explicit, its identifier inferred from its content.
*/
export type ReferenceKindCollapsed = "collapsed";
/**
 The reference is explicit, its identifier explicitly set.
*/
export type ReferenceKindFull = "full";
/**
 The reference is implicit, its identifier inferred from its content.
*/
export type ReferenceKindShortcut = "shortcut";
/**
 format of register payload
*/
export type RegisterPayload = {
    email: string;
    passwordHash: string;
    username: Username;
};
/**
 Document.

 ```markdown
 > | a
     ^
 ```
*/
export type Root = {
    children: Node[];
};
/**
 执行的结果状态，只是一个初步的分析，适用于绝大多数情况
*/
export type SandboxStatus = (
    SandboxStatusMemoryLimitExceeded
    | SandboxStatusOk
    | SandboxStatusRuntimeError
    | SandboxStatusTimeLimitExceeded
);
/**
 超出内存限制
*/
export type SandboxStatusMemoryLimitExceeded = "MemoryLimitExceeded";
/**
 All Correct
*/
export type SandboxStatusOk = "Ok";
/**
 with status code
*/
export type SandboxStatusRuntimeError = {
    RuntimeError: number;
};
/**
 超出时间限制
*/
export type SandboxStatusTimeLimitExceeded = "TimeLimitExceeded";
/**
 一个带类型的 buffer
*/
export type SourceFile = {
    file_type: FileType;
    source: string;
};
/**
*/
export type Statement = {
    meta: StmtMeta;
    statement: Node;
    title: string;
};
/**
*/
export type StmtAssetQuery = {
    id: number;
    name: string;
};
/**
 题目显示时的元数据，在渲染 pdf 题面时也会需要
*/
export type StmtMeta = {
    kind: ( undefined | ProblemKind | null );
    memory: ( undefined | Memory | null );
    time: ( undefined | Elapse | null );
};
/**
*/
export type StmtQuery = {
    id: number;
};
/**
 Strong.

 ```markdown
 > | **a**
     ^^^^^
 ```
*/
export type Strong = {
    children: Node[];
};
/**
*/
export type SubmInfo = {
    meta: SubmMeta;
    raw: SubmRaw;
    report: ( undefined | FullJudgeReport | null );
};
/**
*/
export type SubmMeta = {
    id: number;
    judge_time: ( undefined | null | string );
    lang: ( undefined | FileType | null );
    memory: ( undefined | Memory | null );
    pid: number;
    problem_title: string;
    status: ( undefined | JudgerStatus | null );
    submit_time: string;
    time: ( undefined | Elapse | null );
    uid: number;
    username: Username;
};
/**
*/
export type SubmMetasQuery = {
    lang: ( undefined | FileType | null );
    max_count: number;
    offset: number;
    pid: ( undefined | null | number );
    uid: ( undefined | null | number );
};
/**
 Raw content of user submission is stored on file system.
 This struct provides entries of files in the submission.
*/
export type SubmRaw = Record<string, SourceFile>;
/**
*/
export type SubtaskReport = {
    meta: TaskMeta;
    tasks: ( undefined | TaskReport | null )[];
    total_score: number;
};
/**
 GFM: table.

 ```markdown
 > | | a |
     ^^^^^
 > | | - |
     ^^^^^
 ```
*/
export type Table = {
    align: AlignKind[];
    children: Node[];
};
/**
 GFM: table cell.

 ```markdown
 > | | a |
     ^^^^^
 ```
*/
export type TableCell = {
    children: Node[];
};
/**
 GFM: table row.

 ```markdown
 > | | a |
     ^^^^^
 ```
*/
export type TableRow = {
    children: Node[];
};
/**
 一个测试点的测试结果指标
*/
export type TaskMeta = {
    memory: Memory;
    score_rate: number;
    status: JudgerStatus;
    time: Elapse;
};
/**
 一个测试点的测试结果
*/
export type TaskReport = {
    meta: TaskMeta;
    payload: [ string, TruncStr ][];
};
/**
 Text.

 ```markdown
 > | a
     ^
 ```
*/
export type Text = {
    value: string;
};
/**
 Thematic break.

 ```markdown
 > | ***
     ^^^
 ```
*/
export type ThematicBreak = {
};
/**
 Frontmatter: toml.

 ```markdown
 > | +++
     ^^^
 > | a: b
     ^^^^
 > | +++
     ^^^
 ```
*/
export type Toml = {
    value: string;
};
/**
 裁剪过的文本内容，用于提交记录中文本文件的展示
*/
export type TruncStr = {
    limit: number;
    str: string;
    truncated: number;
};
/**
 拓展语法：两栏布局
 主要用于样例的显示
*/
export type TwoColumns = {
    left: Node;
    right: Node;
};
/**
*/
export type UserDisplayInfo = {
    email: string;
    gender: Gender;
    id: number;
    motto: string;
    name: string;
    register_time: string;
    username: Username;
};
/**
*/
export type UserEditInfo = {
    email: string;
    gender: Gender;
    id: number;
    motto: string;
    name: string;
    register_time: string;
    username: string;
};
/**
*/
export type UserUpdateInfo = {
    email: ( undefined | null | string );
    gender: ( undefined | Gender | null );
    motto: ( undefined | null | string );
    name: ( undefined | null | string );
    password_hash: ( undefined | null | string );
};
/**
 用户名类型，在创建时会进行内容检查，确保没有不合法字符
*/
export type Username = string;
/**
 Frontmatter: yaml.

 ```markdown
 > | ---
     ^^^
 > | a: b
     ^^^^
 > | ---
     ^^^
 ```
*/
export type Yaml = {
    value: string;
};

export function useAPI () {
    return {
        auth: {
            login: {
                /**
                 用户登陆，需要提供用户名和密码的哈希值
                
                 如果登陆成功，http 请求头中会返回 cookie
                 */
                post: { 
                    use: (payload: AuthLoginPostPayload | Ref<AuthLoginPostPayload>) => callAPI("post", "/auth/login", payload) as Promise<ExtAsyncData<void>>,
                    fetch: (payload: AuthLoginPostPayload | Ref<AuthLoginPostPayload>) => fetchAPI("post", "/auth/login", payload) as Promise<void>,
                    key: "/auth/login:post",
                },
            },
            logout: {
                post: {
                    use: () => callAPI("post", "/auth/logout") as Promise<ExtAsyncData<void>>,
                    fetch: () => fetchAPI("post", "/auth/logout") as Promise<void>,
                    key: "/auth/logout:post",
                },
            },
            register: {
                /**
                 */
                post: { 
                    use: (payload: AuthRegisterPostPayload | Ref<AuthRegisterPostPayload>) => callAPI("post", "/auth/register", payload) as Promise<ExtAsyncData<void>>,
                    fetch: (payload: AuthRegisterPostPayload | Ref<AuthRegisterPostPayload>) => fetchAPI("post", "/auth/register", payload) as Promise<void>,
                    key: "/auth/register:post",
                },
            },
            info: {
                get: {
                    use: () => callAPI("get", "/auth/info") as Promise<ExtAsyncData<AuthInfoGetReturn | null>>,
                    fetch: () => fetchAPI("get", "/auth/info") as Promise<AuthInfoGetReturn>,
                    key: "/auth/info:get",
                },
            },
        },
        custom_test: {
            get: {
                use: () => callAPI("get", "/custom_test") as Promise<ExtAsyncData<CustomTestGetReturn | null>>,
                fetch: () => fetchAPI("get", "/custom_test") as Promise<CustomTestGetReturn>,
                key: "/custom_test:get",
            },
            /**
             */
            post: { 
                use: (payload: CustomTestPostPayload | Ref<CustomTestPostPayload>) => callAPI("post", "/custom_test", payload) as Promise<ExtAsyncData<CustomTestPostReturn | null>>,
                fetch: (payload: CustomTestPostPayload | Ref<CustomTestPostPayload>) => fetchAPI("post", "/custom_test", payload) as Promise<CustomTestPostReturn>,
                key: "/custom_test:post",
            },
        },
        problem: {
            metas: {
                /**
                 获取题目列表。
                 后端的 `max_count` 为 u8 类型，限制了此 API 返回的题目数最多为 255 个
                 */
                get: { 
                    use: (payload: ProblemMetasGetPayload | Ref<ProblemMetasGetPayload>) => callAPI("get", "/problem/metas", payload) as Promise<ExtAsyncData<ProblemMetasGetReturn | null>>,
                    fetch: (payload: ProblemMetasGetPayload | Ref<ProblemMetasGetPayload>) => fetchAPI("get", "/problem/metas", payload) as Promise<ProblemMetasGetReturn>,
                    key: "/problem/metas:get",
                },
            },
            statement: {
                /**
                 获取题面数据
                 */
                get: { 
                    use: (payload: ProblemStatementGetPayload | Ref<ProblemStatementGetPayload>) => callAPI("get", "/problem/statement", payload) as Promise<ExtAsyncData<ProblemStatementGetReturn | null>>,
                    fetch: (payload: ProblemStatementGetPayload | Ref<ProblemStatementGetPayload>) => fetchAPI("get", "/problem/statement", payload) as Promise<ProblemStatementGetReturn>,
                    key: "/problem/statement:get",
                },
            },
            fulldata: {
                /**
                 上传题目数据
                 */
                post: { 
                    use: (payload: ProblemFulldataPostPayload | Ref<ProblemFulldataPostPayload>) => callAPI("post", "/problem/fulldata", payload) as Promise<ExtAsyncData<ProblemFulldataPostReturn | null>>,
                    fetch: (payload: ProblemFulldataPostPayload | Ref<ProblemFulldataPostPayload>) => fetchAPI("post", "/problem/fulldata", payload) as Promise<ProblemFulldataPostReturn>,
                    key: "/problem/fulldata:post",
                },
            },
            fulldata_meta: {
                /**
                 题目数据元信息
                 */
                get: { 
                    use: (payload: ProblemFulldataMetaGetPayload | Ref<ProblemFulldataMetaGetPayload>) => callAPI("get", "/problem/fulldata_meta", payload) as Promise<ExtAsyncData<ProblemFulldataMetaGetReturn | null>>,
                    fetch: (payload: ProblemFulldataMetaGetPayload | Ref<ProblemFulldataMetaGetPayload>) => fetchAPI("get", "/problem/fulldata_meta", payload) as Promise<ProblemFulldataMetaGetReturn>,
                    key: "/problem/fulldata_meta:get",
                },
            },
            submit: {
                /**
                 评测题目
                 */
                post: { 
                    use: (payload: ProblemSubmitPostPayload | Ref<ProblemSubmitPostPayload>) => callAPI("post", "/problem/submit", payload) as Promise<ExtAsyncData<ProblemSubmitPostReturn | null>>,
                    fetch: (payload: ProblemSubmitPostPayload | Ref<ProblemSubmitPostPayload>) => fetchAPI("post", "/problem/submit", payload) as Promise<ProblemSubmitPostReturn>,
                    key: "/problem/submit:post",
                },
            },
            statement_assets: {
                /**
                 获取某个题目的附加文件，如果不存在就去获取全局的附加文件
                 */
                get: { 
                    use: (payload: ProblemStatementAssetsGetPayload | Ref<ProblemStatementAssetsGetPayload>) => callAPI("get", "/problem/statement_assets", payload) as Promise<ExtAsyncData<ProblemStatementAssetsGetReturn | null>>,
                    fetch: (payload: ProblemStatementAssetsGetPayload | Ref<ProblemStatementAssetsGetPayload>) => fetchAPI("get", "/problem/statement_assets", payload) as Promise<ProblemStatementAssetsGetReturn>,
                    key: "/problem/statement_assets:get",
                },
            },
        },
        submission: {
            detail: {
                /**
                 查询提交记录
                 */
                get: { 
                    use: (payload: SubmissionDetailGetPayload | Ref<SubmissionDetailGetPayload>) => callAPI("get", "/submission/detail", payload) as Promise<ExtAsyncData<SubmissionDetailGetReturn | null>>,
                    fetch: (payload: SubmissionDetailGetPayload | Ref<SubmissionDetailGetPayload>) => fetchAPI("get", "/submission/detail", payload) as Promise<SubmissionDetailGetReturn>,
                    key: "/submission/detail:get",
                },
            },
            metas: {
                /**
                 获取提交记录列表
                 */
                get: { 
                    use: (payload: SubmissionMetasGetPayload | Ref<SubmissionMetasGetPayload>) => callAPI("get", "/submission/metas", payload) as Promise<ExtAsyncData<SubmissionMetasGetReturn | null>>,
                    fetch: (payload: SubmissionMetasGetPayload | Ref<SubmissionMetasGetPayload>) => fetchAPI("get", "/submission/metas", payload) as Promise<SubmissionMetasGetReturn>,
                    key: "/submission/metas:get",
                },
            },
        },
        user: {
            /**
             */
            get: { 
                use: (payload: UserGetPayload | Ref<UserGetPayload>) => callAPI("get", "/user", payload) as Promise<ExtAsyncData<UserGetReturn | null>>,
                fetch: (payload: UserGetPayload | Ref<UserGetPayload>) => fetchAPI("get", "/user", payload) as Promise<UserGetReturn>,
                key: "/user:get",
            },
            edit: {
                get: {
                    use: () => callAPI("get", "/user/edit") as Promise<ExtAsyncData<UserEditGetReturn | null>>,
                    fetch: () => fetchAPI("get", "/user/edit") as Promise<UserEditGetReturn>,
                    key: "/user/edit:get",
                },
                /**
                 */
                post: { 
                    use: (payload: UserEditPostPayload | Ref<UserEditPostPayload>) => callAPI("post", "/user/edit", payload) as Promise<ExtAsyncData<void>>,
                    fetch: (payload: UserEditPostPayload | Ref<UserEditPostPayload>) => fetchAPI("post", "/user/edit", payload) as Promise<void>,
                    key: "/user/edit:post",
                },
            },
            gravatar: {
                /**
                 */
                get: { 
                    use: (payload: UserGravatarGetPayload | Ref<UserGravatarGetPayload>) => callAPI("get", "/user/gravatar", payload) as Promise<ExtAsyncData<void>>,
                    fetch: (payload: UserGravatarGetPayload | Ref<UserGravatarGetPayload>) => fetchAPI("get", "/user/gravatar", payload) as Promise<void>,
                    key: "/user/gravatar:get",
                },
            },
        },
        contest: {
            metas: {
                /**
                 获取比赛列表
                 */
                get: { 
                    use: (payload: ContestMetasGetPayload | Ref<ContestMetasGetPayload>) => callAPI("get", "/contest/metas", payload) as Promise<ExtAsyncData<ContestMetasGetReturn | null>>,
                    fetch: (payload: ContestMetasGetPayload | Ref<ContestMetasGetPayload>) => fetchAPI("get", "/contest/metas", payload) as Promise<ContestMetasGetReturn>,
                    key: "/contest/metas:get",
                },
            },
        },
    };
}
export type AuthInfoGetReturn = AuthInfoRes;
export type AuthLoginPostPayload = LoginPayload;
export type AuthRegisterPostPayload = RegisterPayload;
export type ContestMetasGetPayload = CtstMetasQuery;
export type ContestMetasGetReturn = ContestMeta[];
export type CustomTestGetReturn = CustomTestResult;
export type CustomTestPostPayload = FormData;
export type CustomTestPostReturn = any;
export type ProblemFulldataMetaGetPayload = FullDataMetaQuery;
export type ProblemFulldataMetaGetReturn = any;
export type ProblemFulldataPostPayload = FormData;
export type ProblemFulldataPostReturn = PostDataReturn;
export type ProblemMetasGetPayload = ProbMetasQuery;
export type ProblemMetasGetReturn = ProblemMeta[];
export type ProblemStatementAssetsGetPayload = StmtAssetQuery;
export type ProblemStatementAssetsGetReturn = any;
export type ProblemStatementGetPayload = StmtQuery;
export type ProblemStatementGetReturn = Statement;
export type ProblemSubmitPostPayload = FormData;
export type ProblemSubmitPostReturn = JudgeReturn;
export type SubmissionDetailGetPayload = DetailQuery;
export type SubmissionDetailGetReturn = DetailReturn;
export type SubmissionMetasGetPayload = SubmMetasQuery;
export type SubmissionMetasGetReturn = SubmMeta[];
export type UserEditGetReturn = UserEditInfo;
export type UserEditPostPayload = UserUpdateInfo;
export type UserGetPayload = ProfileQuery;
export type UserGetReturn = UserDisplayInfo;
export type UserGravatarGetPayload = GravatarInfo;


