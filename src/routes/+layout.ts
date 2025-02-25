export const prerender = true;
export const ssr = false;


export async function load({ url }: { url: URL }) {
    return {
        currentPath: url.pathname, 
    };
}
