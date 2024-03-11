
namespace Legacy
{

#define LEGACY_VERTEXBUFFER_SIZE (0x1000)
#define LEGACY_FACEBUFFER_SIZE   (0x400)

enum FaceFlags {
    FACE_FLAG_TEXTURED_3D      = 0,
    FACE_FLAG_TEXTURED_2D      = 1,
    FACE_FLAG_COLORED_3D       = 2,
    FACE_FLAG_COLORED_2D       = 3,
    FACE_FLAG_FADED            = 4,
    FACE_FLAG_TEXTURED_C       = 5,
    FACE_FLAG_TEXTURED_C_BLEND = 6,
    FACE_FLAG_3DSPRITE         = 7
};

enum MatrixTypes {
    MAT_WORLD = 0,
    MAT_VIEW  = 1,
    MAT_TEMP  = 2,
};

struct Matrix {
    int32 values[4][4];
};

struct Vertex {
    int32 x;
    int32 y;
    int32 z;
    int32 u;
    int32 v;
};

struct Face {
    int32 a;
    int32 b;
    int32 c;
    int32 d;
    uint32 color;
    int32 flag;
};

struct DrawListEntry3D {
    int32 faceID;
    int32 depth;
};

extern int32 vertexCount;
extern int32 faceCount;

extern Matrix matFinal;
extern Matrix matWorld;
extern Matrix matView;
extern Matrix matTemp;

extern int32 projectionX;
extern int32 projectionY;

extern int32 faceLineStart[SCREEN_YSIZE];
extern int32 faceLineEnd[SCREEN_YSIZE];

extern int32 faceLineStartU[SCREEN_YSIZE];
extern int32 faceLineEndU[SCREEN_YSIZE];
extern int32 faceLineStartV[SCREEN_YSIZE];
extern int32 faceLineEndV[SCREEN_YSIZE];

extern Vertex vertexBuffer[LEGACY_VERTEXBUFFER_SIZE];
extern Vertex vertexBufferT[LEGACY_VERTEXBUFFER_SIZE];

extern Face faceBuffer[LEGACY_FACEBUFFER_SIZE];

extern DrawListEntry3D drawList3D[LEGACY_FACEBUFFER_SIZE];

extern int32 fogColor;
extern int32 fogStrength;

void SetIdentityMatrix(Matrix *matrix);
void MatrixMultiply(Matrix *matrixA, Matrix *matrixB);
void MatrixInverse(Matrix *matrix);
void MatrixTranslateXYZ(Matrix *Matrix, int32 x, int32 y, int32 z);
void MatrixScaleXYZ(Matrix *matrix, int32 scaleX, int32 scaleY, int32 scaleZ);
void MatrixRotateX(Matrix *matrix, int32 rotationX);
void MatrixRotateY(Matrix *matrix, int32 rotationY);
void MatrixRotateZ(Matrix *matrix, int32 rotationZ);
void MatrixRotateXYZ(Matrix *matrix, int16 rotationX, int16 rotationY, int16 rotationZ);
void ProcessScanEdge(Vertex *vertA, Vertex *vertB);
void ProcessScanEdgeUV(Vertex *vertA, Vertex *vertB);
void TransformVertexBuffer();
void TransformVertices(Matrix *matrix, int32 startIndex, int32 endIndex);
void Sort3DDrawList();
void Draw3DScene(int32 spriteSheetID);

} // namespace Legacy