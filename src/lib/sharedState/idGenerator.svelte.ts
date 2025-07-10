let nextID: number = $state(0);

function createInstanceId(): number {
  nextID += 1;
  return nextID - 1;
}

export { createInstanceId };
