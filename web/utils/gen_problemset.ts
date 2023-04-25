import { faker } from "@faker-js/faker";

export function genProbMeta() {
  const id = faker.datatype.number({
    max: 99,
  });
  return {
    id,
    title: faker.music.songName(),
    link: "/problem/" + id,
    accepts: faker.datatype.number({ max: 1000 }),
    accepted: faker.datatype.boolean(), // false or undefined
  };
}

export function genProbSet(len: number) {
  faker.seed(0);
  const lst = [];
  for (let i = 0; i < len; i++) {
    lst.push(genProbMeta());
  }
  lst.sort((a, b) => a.id - b.id);
  return lst;
}
