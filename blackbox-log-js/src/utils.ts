export function unreachable(_: never): never {
	throw new Error('reached unreachable');
}
