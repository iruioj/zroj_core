// generated by server/src/bin/gen_docs.rs
// DO NOT EDIT.

import type { AsyncData } from "nuxt/app";
import type { FetchError } from "ofetch";

function callAPI(method: string, path: string, args?: any): Promise<any> {
    if (process.client) {
        console.log('client call api', method, path, args)
    }
    path = useRuntimeConfig().public.apiBase + path;

    const options = {
        server: false, // 这只会降低首次加载的体验
        key: method + ":" + path,
        method: method as any,
        credentials: 'include' as any,
        headers: useRequestHeaders()
    };
    if (args === undefined) {
        return useFetch(path, options);
    } else if (method === 'get') {
        return useFetch(path, { ...options, query: args });
    } else {
        return useFetch(path, { ...options, body: args });
    }
}
type IOKind = ("StdIO"|{FileIO:{input:FileDescriptor;output:FileDescriptor;};});
type StmtMeta = {kind:(ProblemKind|null);memory:(Memory|null);time:(Elapse|null);title:string;};
type ProfileQuery = {username:Username;};
type Delete = {children:Node[];};
type Root = {children:Node[];};
type ThematicBreak = {};
type Break = {};
type ReferenceKind = ("collapsed"|"full"|"shortcut");
type Username = string;
type GravatarInfo = {email:string;no_cache:(null|boolean);};
type StmtQuery = {id:number;};
type Image = {alt:string;title:(null|string);url:string;};
type Code = {lang:(null|string);meta:(null|string);value:string;};
type List = {children:Node[];ordered:boolean;spread:boolean;start:(null|number);};
type ImageReference = {alt:string;identifier:string;label:(null|string);reference_kind:ReferenceKind;};
type ProblemKind = ("Interactive"|"SubmitAnswer"|{Traditional:IOKind;});
type Yaml = {value:string;};
type Memory = number;
type Definition = {identifier:string;label:(null|string);title:(null|string);url:string;};
type BlockQuote = {children:Node[];};
type UserUpdateInfo = {email:(null|string);gender:(null|number);motto:(null|string);name:(null|string);password_hash:(null|string);};
type Paragraph = {children:Node[];};
type TableRow = {children:Node[];};
type Elapse = number;
type Heading = {children:Node[];depth:number;};
type UserDisplayInfo = {email:string;gender:number;id:number;motto:string;name:string;register_time:string;username:Username;};
type Math = {meta:(null|string);value:string;};
type Text = {value:string;};
type InlineMath = {value:string;};
type ListItem = {checked:(null|boolean);children:Node[];spread:boolean;};
type AuthInfoRes = {email:string;username:Username;};
type Emphasis = {children:Node[];};
type Table = {align:AlignKind[];children:Node[];};
type UserEditInfo = {email:string;gender:number;id:number;motto:string;name:string;register_time:string;username:string;};
type FootnoteDefinition = {children:Node[];identifier:string;label:(null|string);};
type LinkReference = {children:Node[];identifier:string;label:(null|string);reference_kind:ReferenceKind;};
type RegisterPayload = {email:string;passwordHash:string;username:Username;};
type Statement = {meta:StmtMeta;statement:Node;};
type AlignKind = ("center"|"left"|"none"|"right");
type Node = ((Delete&{type:"delete";})|(Root&{type:"root";})|(ThematicBreak&{type:"thematicBreak";})|(Break&{type:"break";})|(Image&{type:"image";})|(Code&{type:"code";})|(List&{type:"list";})|(ImageReference&{type:"imageReference";})|(Yaml&{type:"yaml";})|(Definition&{type:"definition";})|(BlockQuote&{type:"blockquote";})|(Paragraph&{type:"paragraph";})|(TableRow&{type:"tableRow";})|(Heading&{type:"heading";})|(Math&{type:"math";})|(Text&{type:"text";})|(InlineMath&{type:"inlineMath";})|(ListItem&{type:"listItem";})|(Emphasis&{type:"emphasis";})|(Table&{type:"table";})|(FootnoteDefinition&{type:"footnoteDefinition";})|(LinkReference&{type:"linkReference";})|(TableCell&{type:"tableCell";})|(Toml&{type:"toml";})|(InlineCode&{type:"inlineCode";})|(FootnoteReference&{type:"footnoteReference";})|(Strong&{type:"strong";})|(Html&{type:"html";})|(TwoColumns&{type:"twoColumns";})|(Link&{type:"link";}));
type TableCell = {children:Node[];};
type MetasQuery = {max_count:number;max_id:(null|number);min_id:(null|number);pattern:(null|string);};
type PostDataReturn = {id:number;};
type Toml = {value:string;};
type InlineCode = {value:string;};
type FileDescriptor = ("Stdin"|"Stdout"|{Named:string;});
type LoginPayload = {passwordHash:string;username:Username;};
type FootnoteReference = {identifier:string;label:(null|string);};
type Strong = {children:Node[];};
type Html = {value:string;};
type TwoColumns = {left:Node;right:Node;};
type Link = {children:Node[];title:(null|string);url:string;};
export function useAPI () { return { auth: { login: { post: (payload: LoginPayload) => callAPI("post", "/auth/login", payload) as Promise<AsyncData<void, FetchError>>,
 },
logout: { post: () => callAPI("post", "/auth/logout") as Promise<AsyncData<void, FetchError>>,
 },
register: { post: (payload: RegisterPayload) => callAPI("post", "/auth/register", payload) as Promise<AsyncData<void, FetchError>>,
 },
info: { get: () => callAPI("get", "/auth/info") as Promise<AsyncData<AuthInfoGetReturn | null, FetchError>>,
 },
 },
user: { get: (payload: ProfileQuery) => callAPI("get", "/user", payload) as Promise<AsyncData<UserGetReturn | null, FetchError>>,
edit: { get: () => callAPI("get", "/user/edit") as Promise<AsyncData<UserEditGetReturn | null, FetchError>>,
post: (payload: UserUpdateInfo) => callAPI("post", "/user/edit", payload) as Promise<AsyncData<void, FetchError>>,
 },
gravatar: { get: (payload: GravatarInfo) => callAPI("get", "/user/gravatar", payload) as Promise<AsyncData<void, FetchError>>,
 },
 },
problem: { full_dbg: { get: () => callAPI("get", "/problem/full_dbg") as Promise<AsyncData<ProblemFullDbgGetReturn | null, FetchError>>,
 },
metas: { get: (payload: MetasQuery) => callAPI("get", "/problem/metas", payload) as Promise<AsyncData<ProblemMetasGetReturn | null, FetchError>>,
 },
statement: { get: (payload: StmtQuery) => callAPI("get", "/problem/statement", payload) as Promise<AsyncData<ProblemStatementGetReturn | null, FetchError>>,
 },
fulldata: { post: () => callAPI("post", "/problem/fulldata") as Promise<AsyncData<ProblemFulldataPostReturn | null, FetchError>>,
 },
 },
 }; }
export type AuthInfoGetReturn = AuthInfoRes;
export type AuthLoginPostPayload = LoginPayload;
export type AuthRegisterPostPayload = RegisterPayload;
export type ProblemFullDbgGetReturn = [number,StmtMeta][];
export type ProblemFulldataPostReturn = PostDataReturn;
export type ProblemMetasGetPayload = MetasQuery;
export type ProblemMetasGetReturn = [number,StmtMeta][];
export type ProblemStatementGetPayload = StmtQuery;
export type ProblemStatementGetReturn = Statement;
export type UserEditGetReturn = UserEditInfo;
export type UserEditPostPayload = UserUpdateInfo;
export type UserGetPayload = ProfileQuery;
export type UserGetReturn = UserDisplayInfo;
export type UserGravatarGetPayload = GravatarInfo;

