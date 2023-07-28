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
export function useAPI () { return { auth: { login: { post: (payload: {username: string;passwordHash: string;}) => callAPI("post", "/auth/login", payload) as Promise<AsyncData<void, FetchError>>,
 },
logout: { post: () => callAPI("post", "/auth/logout") as Promise<AsyncData<void, FetchError>>,
 },
register: { post: (payload: {email: string;username: string;passwordHash: string;}) => callAPI("post", "/auth/register", payload) as Promise<AsyncData<void, FetchError>>,
 },
info: { get: () => callAPI("get", "/auth/info") as Promise<AsyncData<AuthInfoGetReturn | null, FetchError>>,
 },
 },
user: { get: (payload: {username: string;}) => callAPI("get", "/user", payload) as Promise<AsyncData<UserGetReturn | null, FetchError>>,
edit: { get: () => callAPI("get", "/user/edit") as Promise<AsyncData<UserEditGetReturn | null, FetchError>>,
post: (payload: {password_hash?: string;email?: string;motto?: string;name?: string;gender?: "Male" | "Female" | "Others" | "Private";}) => callAPI("post", "/user/edit", payload) as Promise<AsyncData<void, FetchError>>,
 },
 },
problem: { full_dbg: { get: () => callAPI("get", "/problem/full_dbg") as Promise<AsyncData<ProblemFullDbgGetReturn | null, FetchError>>,
 },
metas: { get: (payload: {max_count: number;pattern?: string;min_id?: number;max_id?: number;}) => callAPI("get", "/problem/metas", payload) as Promise<AsyncData<ProblemMetasGetReturn | null, FetchError>>,
 },
statement: { get: (payload: {id: number;}) => callAPI("get", "/problem/statement", payload) as Promise<AsyncData<ProblemStatementGetReturn | null, FetchError>>,
 },
 },
 }; }
export type AuthInfoGetReturn = {username: string;email: string;};
export type AuthLoginPostPayload = {username: string;passwordHash: string;};
export type AuthRegisterPostPayload = {email: string;username: string;passwordHash: string;};
export type ProblemFullDbgGetReturn = [number,{title: string;time?: number;memory?: number;kind?: {Traditional: "StdIO" | {FileIO: {input: "Stdin" | "Stdout" | {Named: string};output: "Stdin" | "Stdout" | {Named: string};}}} | "Interactive" | "SubmitAnswer";}][];
export type ProblemMetasGetPayload = {max_count: number;pattern?: string;min_id?: number;max_id?: number;};
export type ProblemMetasGetReturn = [number,{title: string;time?: number;memory?: number;kind?: {Traditional: "StdIO" | {FileIO: {input: "Stdin" | "Stdout" | {Named: string};output: "Stdin" | "Stdout" | {Named: string};}}} | "Interactive" | "SubmitAnswer";}][];
export type ProblemStatementGetPayload = {id: number;};
export type ProblemStatementGetReturn = { statement: any; meta: {title: string;time?: number;memory?: number;kind?: {Traditional: "StdIO" | {FileIO: {input: "Stdin" | "Stdout" | {Named: string};output: "Stdin" | "Stdout" | {Named: string};}}} | "Interactive" | "SubmitAnswer";};};
export type UserEditGetReturn = {id: number;username: string;email: string;motto: string;name: string;register_time: string;gender: "Male" | "Female" | "Others" | "Private";};
export type UserEditPostPayload = {password_hash?: string;email?: string;motto?: string;name?: string;gender?: "Male" | "Female" | "Others" | "Private";};
export type UserGetPayload = {username: string;};
export type UserGetReturn = {id: number;username: string;email: string;motto: string;name: string;register_time: string;gender: "Male" | "Female" | "Others" | "Private";};

