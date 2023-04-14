import { faker } from "@faker-js/faker";

export function genProbMeta() {
  const id = faker.datatype.number({
    max: 99,
  });
  const like = faker.datatype.boolean();
  return {
    id,
    title: faker.music.songName(),
    link: "/problem/" + id,
    totalLikes: faker.datatype.number(), // likes - dislikes
    like,
    dislike: !like && faker.datatype.boolean(),
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
